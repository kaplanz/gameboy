use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::{Parser, ValueHint};
use color_eyre::eyre::{Result, WrapErr};
use gameboy::core::Emulator;
use gameboy::dmg::cart::{Cartridge, Header};
use gameboy::dmg::{Button, GameBoy, Screen, SCREEN};
use log::{debug, info};
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use remus::Machine;

use crate::pal::Palette;

mod pal;

/// Game Boy emulator written in Rust.
#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Cartridge ROM image file.
    #[clap(parse(from_os_str))]
    #[clap(value_hint = ValueHint::FilePath)]
    rom: PathBuf,

    /// Check ROM integrity.
    #[clap(long = "check")]
    #[clap(short = 'c')]
    chk: bool,

    /// Color palette.
    #[clap(default_value_t)]
    #[clap(long = "palette")]
    pal: Palette,
}

fn main() -> Result<()> {
    // Install panic and error report handlers
    color_eyre::install()?;
    // Initialize logger
    env_logger::init();
    // Parse args
    let args = Args::parse();

    // Read the ROM
    let rom = {
        // Open ROM file
        let f = File::open(&args.rom)
            .with_context(|| format!("failed to open ROM: `{}`", args.rom.display()))?;
        // Read ROM into a buffer
        let mut buf = Vec::new();
        // NOTE: Game Paks manufactured by Nintendo have a maximum 8 MiB ROM.
        let read = f
            .take(0x800000)
            .read_to_end(&mut buf)
            .with_context(|| format!("failed to read ROM: `{}`", args.rom.display()))?;
        info!("Read {read} bytes");

        buf
    };
    // Check ROM integrity
    if args.chk {
        Header::check(&rom).with_context(|| "failed ROM integrity check")?;
    }

    // Initialize the cartridge
    let cart = Cartridge::new(&rom)
        .with_context(|| format!("failed to load cartridge: `{}`", args.rom.display()))?;
    // Extract ROM title from cartridge
    let title = match cart.header().title.replace('\0', " ").trim() {
        "" => "Game Boy",
        title => title,
    }
    .to_string();

    // Create emulator instance
    let mut emu = GameBoy::new(cart);

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

    // Mark the starting time
    let mut now = std::time::Instant::now();
    let mut active = 0;

    // TODO: Run emulator on a 4 MiHz clock
    while win.is_open() {
        // Perform a single cycle
        emu.cycle();

        // Redraw the screen (if needed)
        emu.redraw(|screen: &Screen| {
            let buf: Vec<_> = screen.iter().map(|&pix| args.pal[pix as usize]).collect();
            win.update_with_buffer(&buf, SCREEN.width, SCREEN.height)
                .unwrap()
        });

        // Send joypad input
        #[rustfmt::skip]
        let keys: Vec<_> = win.get_keys().into_iter().flat_map(|key| match key {
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
        emu.send(keys);

        // Calculate real-time clock frequency
        if now.elapsed().as_secs() > 0 {
            debug!("Frequency: {active}");
            active = 0;
            now = std::time::Instant::now();
        }
        active += 1;
    }

    Ok(())
}
