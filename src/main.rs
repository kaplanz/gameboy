use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, ValueHint};
use gameboy::{Cartridge, Emulator, GameBoy, SCREEN};
use log::info;
use minifb::{Scale, ScaleMode, Window, WindowOptions};
use remus::{clk, Machine};

const WIDTH: usize = SCREEN.0;
const HEIGHT: usize = SCREEN.1;

/// Game Boy emulator written in Rust.
#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Cartridge ROM image file
    #[clap(parse(from_os_str))]
    #[clap(value_hint = ValueHint::FilePath)]
    rom: PathBuf,
}

fn main() -> anyhow::Result<()> {
    // Initialize logger
    env_logger::init();
    // Parse args
    let args = Args::parse();

    // Read the ROM
    let rom = {
        // Open ROM file
        let f = File::open(&args.rom).with_context(|| "Failed to open ROM file.".to_string())?;
        // Read ROM into a buffer
        let mut buf = Vec::new();
        // NOTE: Game Paks manufactured by Nintendo have a maximum 8 MiB ROM.
        f.take(0x800000)
            .read_to_end(&mut buf)
            .with_context(|| "Failed to read ROM file.".to_string())?;

        buf
    };
    // Initialize the cartridge
    let cart = Cartridge::new(&rom).with_context(|| "Failed to parse ROM header.".to_string())?;
    // Extract ROM title from cartridge
    let title = match cart.header().title.replace('\0', " ").trim() {
        "" => "Game Boy",
        title => title,
    }
    .to_string();
    // Create emulator instance
    let mut gb = GameBoy::new(cart);

    // Set up emulator for running
    gb.setup();
    // Create a framebuffer window
    let mut win = Window::new(
        &title,
        WIDTH,
        HEIGHT,
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
    // Run emulator on a 4 MiHz clock
    for _ in clk::with_freq(4194304) {
        // Perform a single cycle
        gb.cycle();

        // Redraw the screen (if needed)
        gb.redraw(|buf| win.update_with_buffer(buf, WIDTH, HEIGHT).unwrap());

        // Calculate real-time clock frequency
        if now.elapsed().as_secs() > 0 {
            info!("Frequency: {active}");
            active = 0;
            now = std::time::Instant::now();
        }
        active += 1;
    }

    Ok(())
}
