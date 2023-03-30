use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::{Parser, ValueEnum, ValueHint};
use color_eyre::eyre::{Result, WrapErr};
use gameboy::dmg::cart::{Cartridge, Header};
use gameboy::dmg::{BootRom, GameBoy, SCREEN};
use log::{info, warn};
use minifb::{Scale, ScaleMode, Window, WindowOptions};

use crate::app::{App, Opts};
use crate::pal::Palette;

mod app;
mod pal;

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

    // Read the boot ROM
    let boot = boot(args.boot)?;
    // Read the cartridge
    let cart = cart(args.rom, args.chk, args.force)?;
    let title = match cart.header().title.replace('\0', " ").trim() {
        "" => "Game Boy",
        title => title,
    } // extract title from cartridge
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
    // Create main window
    let win = Window::new(&title, SCREEN.width, SCREEN.height, opts).unwrap();

    // Create debug info
    let debug = if args.debug {
        Some(app::Debug::new(opts))
    } else {
        None
    };

    // Prepare app, options
    let Args { pal, speed, .. } = args;
    let opts = Opts { title, pal, speed };
    let app = App {
        opts,
        emu,
        win,
        debug,
    };

    // Run the app
    app.run();

    Ok(())
}

fn boot(path: Option<PathBuf>) -> Result<Option<BootRom>> {
    // Prepare the boot ROM
    let boot = path
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

    Ok(boot)
}

fn cart(path: Option<PathBuf>, chk: bool, force: bool) -> Result<Cartridge> {
    if let Some(path) = path {
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
        if chk {
            Header::check(&rom).with_context(|| "failed ROM integrity check")?;
            info!("Passed ROM integrity check");
        }

        // Initialize the cartridge
        let cart = if force {
            // Force cartridge from ROM
            Cartridge::unchecked(&rom)
        } else {
            // Exit on cartridge failure
            Cartridge::new(&rom)
                .with_context(|| format!("failed to load cartridge: `{}`", path.display()))?
        };
        info!("Loaded cartridge:\n{}", cart.header());

        Ok(cart)
    } else {
        warn!("Missing cartridge; defaulting to blank");
        Ok(Cartridge::blank())
    }
}
