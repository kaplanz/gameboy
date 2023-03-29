#![allow(clippy::too_many_lines)]

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::{Parser, ValueEnum, ValueHint};
use color_eyre::eyre::{Result, WrapErr};
use gameboy::core::Emulator;
use gameboy::dmg::cart::{Cartridge, Header};
use gameboy::dmg::{BootRom, Button, GameBoy, Screen, SCREEN};
use gameboy_core::Tile;
use itertools::Itertools;
use log::{debug, info, warn};
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use remus::{Clock, Machine};

use crate::palette::Palette;

mod palette;

/// Game Boy main clock frequency, set to 4,194,304 Hz.
const FREQ: u32 = 0x0040_0000;

/// Emulation speed modifier.
#[derive(Clone, Debug, Default, PartialEq, ValueEnum)]
pub enum Speed {
    Half,
    #[default]
    Full,
    Double,
    Triple,
    Max,
    #[clap(skip)]
    Custom(u32),
}

/// Game Boy emulator written in Rust.
#[allow(clippy::struct_excessive_bools)]
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Cartridge ROM image file.
    #[arg(value_hint = ValueHint::FilePath)]
    rom: Option<PathBuf>,

    /// Boot ROM image file.
    #[arg(short, long)]
    #[arg(value_hint = ValueHint::FilePath)]
    boot: Option<PathBuf>,

    /// Check ROM integrity.
    ///
    /// Verifies that both the header and global checksums match the data within
    /// the ROM.
    #[arg(short, long = "check")]
    chk: bool,

    /// Force cartridge construction.
    ///
    /// Causes the cartridge generation to always succeed, even if the ROM does
    /// not contain valid data.
    #[arg(short, long)]
    force: bool,

    /// Exit after loading cartridge.
    ///
    /// Instead of entering the main emulation loop, return immediately after
    /// loading the cartridge ROM. This option could be used along with
    /// `--check` to validate a ROM, or using logging to print the cartridge
    /// header without actually performing any emulation.
    #[arg(short = 'x', long)]
    exit: bool,

    /// Launch in debug mode.
    ///
    /// Causes the emulator to run in debug mode. Provided debugging options
    /// include rendering the PPU's video RAM contents.
    #[arg(long)]
    debug: bool,

    /// DMG-01 color palette.
    ///
    /// Defines the 2-bit color palette for the DMG-01 Game Boy model. The
    /// palette must be specified as a list of hex color values from lightest to
    /// darkest.
    #[arg(default_value_t)]
    #[arg(long = "palette")]
    pal: Palette,

    /// Run at full-speed.
    ///
    /// Causes the emulator to run at the maximum possible speed the host
    /// machine supports.
    #[arg(short, long)]
    #[arg(value_enum, default_value_t = Speed::Full)]
    speed: Speed,
}

fn main() -> Result<()> {
    // Install panic and error report handlers
    color_eyre::install()?;
    // Initialize logger
    env_logger::init();
    // Parse args
    let args = Args::parse();

    // Prepare the boot ROM
    let boot = args
        .boot
        .map(|boot| -> Result<_> {
            // Open boot ROM file
            let mut f = File::open(&boot)
                .with_context(|| format!("failed to open boot ROM: `{}`", boot.display()))?;
            // Read boot ROM into a buffer
            // NOTE: Boot ROM is exactly 256 bytes.
            let mut buf = [0u8; 0x0100];
            f.read_exact(&mut buf)
                .with_context(|| format!("failed to read full boot ROM: `{}`", boot.display()))?;
            info!(
                "Read {} bytes from boot ROM: `{}`",
                buf.len(),
                boot.display(),
            );

            Ok(buf)
        })
        .transpose()?;
    // Initialize the boot rom
    let boot = boot.as_ref().map(BootRom::from);

    // Prepare the cartridge
    let cart = if let Some(path) = args.rom {
        let rom = {
            // Open ROM file
            let f = File::open(&path)
                .with_context(|| format!("failed to open ROM: `{}`", path.display()))?;
            // Read ROM into a buffer
            // NOTE: Game Paks manufactured by Nintendo have a maximum 8 MiB ROM.
            let mut buf = Vec::new();
            let read = f
                .take(0x0080_0000)
                .read_to_end(&mut buf)
                .with_context(|| format!("failed to read ROM: `{}`", path.display()))?;
            info!("Read {read} bytes from ROM: `{}`", path.display());

            buf
        };

        // Check ROM integrity
        if args.chk {
            Header::check(&rom).with_context(|| "failed ROM integrity check")?;
            info!("Passed ROM integrity check");
        }

        // Initialize the cartridge
        let cart = if args.force {
            // Force cartridge from ROM
            Cartridge::unchecked(&rom)
        } else {
            // Exit on cartridge failure
            Cartridge::new(&rom)
                .with_context(|| format!("failed to load cartridge: `{}`", path.display()))?
        };
        info!("Loaded cartridge:\n{}", cart.header());

        cart
    } else {
        warn!("Missing cartridge; defaulting to blank");
        Cartridge::blank()
    };
    // Extract ROM title from cartridge
    let title = match cart.header().title.replace('\0', " ").trim() {
        "" => "Game Boy",
        title => title,
    }
    .to_string();

    // Create emulator instance
    let mut emu = if let Some(boot) = boot {
        GameBoy::with(boot)
    } else {
        GameBoy::new()
    };
    // Load the cartridge into the emulator
    emu.load(cart);

    // Exit after loading
    if args.exit {
        return Ok(());
    }

    // Define window options
    let opts = WindowOptions {
        resize: true,
        scale: Scale::X2,
        scale_mode: ScaleMode::AspectRatioStretch,
        ..Default::default()
    };
    // Create debug mode windows
    let mut debug = if args.debug {
        Some(Debug {
            tdat: Window::new("Tile Data", 16 * 8, 24 * 8, opts).unwrap(),
            map1: Window::new(
                "Tile Map 1",
                32 * 8,
                32 * 8,
                WindowOptions {
                    scale: Scale::X1,
                    ..opts
                },
            )
            .unwrap(),
            map2: Window::new(
                "Tile Map 2",
                32 * 8,
                32 * 8,
                WindowOptions {
                    scale: Scale::X1,
                    ..opts
                },
            )
            .unwrap(),
        })
    } else {
        None
    };
    // Create a framebuffer window
    let mut win = Window::new(&title, SCREEN.width, SCREEN.height, opts).unwrap();

    // Create 4 MiHz clock to sync emulator
    let divider = 0x100; // user a clock divider to sync
    let freq = match args.speed {
        Speed::Half => FREQ / 2,
        Speed::Full => FREQ,
        Speed::Double => 2 * FREQ,
        Speed::Triple => 3 * FREQ,
        Speed::Max => divider, // special case
        Speed::Custom(freq) => freq,
    };
    let mut clk = Clock::with_freq(freq / divider);

    // Initialize timer, counters
    let mut now = std::time::Instant::now();
    let mut cycles = 0;
    let mut fps = 0;

    // Emulation loop
    while win.is_open() {
        // Calculate wall-clock frequency
        if now.elapsed().as_secs() > 0 {
            // Print cycle stats
            debug!(
                "Frequency: {freq:0.4} MHz ({speedup:.1}%), FPS: {fps} Hz",
                freq = f64::from(cycles) / 1e6,
                speedup = 100. * f64::from(cycles) / f64::from(FREQ)
            );
            // Update the title to display the frequency
            win.set_title(&format!("{title} ({fps} Hz)"));
            // Reset timer, counters
            now = std::time::Instant::now();
            cycles = 0;
            fps = 0;
        }

        // Synchronize with wall-clock
        if cycles % divider == 0 && args.speed != Speed::Max {
            // Delay until clock is ready
            clk.next();
        }

        // Perform a single cycle
        emu.cycle();

        // Redraw the screen (if needed)
        emu.redraw(|screen: &Screen| {
            let buf = screen
                .iter()
                .map(|&pix| args.pal[pix as usize].into())
                .collect_vec();
            win.update_with_buffer(&buf, SCREEN.width, SCREEN.height)
                .unwrap();
            fps += 1; // update frames drawn
        });

        // Update the debug screens every second
        if let Some(debug) = &mut debug {
            if cycles == 0 {
                // Retrieve a copy of the VRAM
                let vram = emu.vram();
                // Extract tile data, maps
                let tiles = vram[..0x1800]
                    .chunks_exact(16) // 16-bytes per tile
                    .map(|tile| Tile::from(<[_; 16]>::try_from(tile).unwrap()))
                    .collect_vec();
                let map1 = vram[0x1800..0x1c00]
                    .iter()
                    .map(|&tnum| tiles[tnum as usize].clone())
                    .collect_vec();
                let map2 = vram[0x1c00..0x2000]
                    .iter()
                    .map(|&tnum| tiles[tnum as usize].clone())
                    .collect_vec();
                // Define rendering function
                let render = |tiles: &[Tile], width: usize| -> Vec<u32> {
                    tiles
                        .chunks_exact(width) // tiles per row
                        .flat_map(|row| {
                            row.iter()
                                .flat_map(|tile| tile.iter().enumerate())
                                .sorted_by_key(|row| row.0)
                                .map(|(_, row)| row)
                                .collect_vec()
                        })
                        .flat_map(|row| row.into_iter().map(|pix| args.pal[pix as usize].into()))
                        .collect_vec()
                };
                // Render tile data
                let tdat = render(tiles.as_slice(), 16);
                debug
                    .tdat
                    .update_with_buffer(&tdat, 16 * 8, 24 * 8)
                    .unwrap();
                // Render tile maps
                let map1 = render(map1.as_slice(), 32);
                debug
                    .map1
                    .update_with_buffer(&map1, 32 * 8, 32 * 8)
                    .unwrap();
                let map2 = render(map2.as_slice(), 32);
                debug
                    .map2
                    .update_with_buffer(&map2, 32 * 8, 32 * 8)
                    .unwrap();
            }
        }

        // Send joypad input (sampled every 64 cycles)
        if cycles % 0x40 == 0 {
            #[rustfmt::skip]
            let keys = win.get_keys().into_iter().filter_map(|key| match key {
                Key::Z     => Some(Button::A),
                Key::X     => Some(Button::B),
                Key::Space => Some(Button::Select),
                Key::Enter => Some(Button::Start),
                Key::Right => Some(Button::Right),
                Key::Left  => Some(Button::Left),
                Key::Up    => Some(Button::Up),
                Key::Down  => Some(Button::Down),
                _ => None
            }).collect_vec();
            emu.send(&keys);
        }

        // Clock another cycle
        cycles += 1;
    }

    Ok(())
}

#[derive(Debug)]
struct Debug {
    tdat: Window,
    map1: Window,
    map2: Window,
}
