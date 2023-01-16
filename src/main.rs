#![allow(clippy::too_many_lines)]

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::{Parser, ValueEnum, ValueHint};
use color_eyre::eyre::{Result, WrapErr};
use gameboy::core::Emulator;
use gameboy::dmg::cart::{Cartridge, Header};
use gameboy::dmg::{BootRom, Button, GameBoy, Screen, SCREEN};
use log::{debug, info};
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use remus::{Clock, Machine};

use crate::palette::Palette;

mod palette;

/// Game Boy main clock frequency, set to 4.194304 Hz.
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
}

/// Game Boy emulator written in Rust.
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

    // Prepare the cartridge
    let cart = if let Some(path) = args.rom {
        // Open ROM file
        let rom = {
            let f = File::open(&path)
                .with_context(|| format!("failed to open ROM: `{}`", path.display()))?;
            // Read ROM into a buffer
            let mut buf = Vec::new();
            // NOTE: Game Paks manufactured by Nintendo have a maximum 8 MiB ROM.
            let read = f
                .take(0x0080_0000)
                .read_to_end(&mut buf)
                .with_context(|| format!("failed to read ROM: `{}`", path.display()))?;
            info!("Read {read} bytes from ROM");

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
        info!("Loaded ROM:\n{}", cart.header());

        cart
    } else {
        Cartridge::blank()
    };
    // Extract ROM title from cartridge
    let title = match cart.header().title.replace('\0', " ").trim() {
        "" => "Game Boy",
        title => title,
    }
    .to_string();

    // Read the boot ROM
    let boot = args
        .boot
        .map(|boot| -> Result<_> {
            // Open boot ROM file
            let f = File::open(&boot)
                .with_context(|| format!("failed to open boot ROM: `{}`", boot.display()))?;
            // Read ROM into a buffer
            let mut buf = Vec::new();
            // NOTE: Game Paks manufactured by Nintendo have a maximum 8 MiB ROM.
            let read = f
                .take(0x0100)
                .read_to_end(&mut buf)
                .with_context(|| format!("failed to read boot ROM: `{}`", boot.display()))?;
            info!("Read {read} bytes from boot ROM");

            Ok(buf)
        })
        .transpose()?;
    // Initialize the boot rom
    let boot = boot.as_deref().map(BootRom::try_from).transpose()?;

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

    // Create a framebuffer window
    let mut win = Window::new(
        &title,
        SCREEN.width,
        SCREEN.height,
        WindowOptions {
            resize: true,
            scale: Scale::X2,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..Default::default()
        },
    )
    .unwrap();

    // Create 4 MiHz clock to sync emulator
    let divider = 0x100;
    let freq = match args.speed {
        Speed::Half => FREQ / 2,
        Speed::Full => FREQ,
        Speed::Double => 2 * FREQ,
        Speed::Triple => 3 * FREQ,
        Speed::Max => divider, // special case
    };
    let mut clk = Clock::with_freq(freq / divider);

    // Initialize timer, counters
    let mut now = std::time::Instant::now();
    let mut cycles = 0;
    let mut fps = 0;

    // Emulation loop
    while win.is_open() {
        // Synchronize with wall-clock
        if cycles % divider == 0 && args.speed != Speed::Max {
            // Delay until clock is ready
            clk.next();
        }

        // Perform a single cycle
        emu.cycle();

        // Redraw the screen (if needed)
        emu.redraw(|screen: &Screen| {
            let buf: Vec<_> = screen
                .iter()
                .map(|&pix| args.pal[pix as usize].into())
                .collect();
            win.update_with_buffer(&buf, SCREEN.width, SCREEN.height)
                .unwrap();
            fps += 1; // update frames drawn
        });

        // Send joypad input (sampled every 64 cycles)
        if cycles % 0x40 == 0 {
            #[rustfmt::skip]
            let keys: Vec<_> = win.get_keys().into_iter().filter_map(|key| match key {
                Key::Z     => Some(Button::A),
                Key::X     => Some(Button::B),
                Key::Space => Some(Button::Select),
                Key::Enter => Some(Button::Start),
                Key::Right => Some(Button::Right),
                Key::Left  => Some(Button::Left),
                Key::Up    => Some(Button::Up),
                Key::Down  => Some(Button::Down),
                _ => None
            }).collect();
            emu.send(&keys);
        }

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

        // Clock another cycle
        cycles += 1;
    }

    Ok(())
}
