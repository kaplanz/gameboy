#![warn(clippy::pedantic)]

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use eyre::{ensure, Result, WrapErr};
use gameboy::dmg::cart::Cartridge;
use gameboy::dmg::{Boot, GameBoy, SCREEN};
use log::{info, warn};
use minifb::{Scale, ScaleMode, Window, WindowOptions};
#[cfg(feature = "gbd")]
use tracing_subscriber::fmt::Layer;
#[cfg(feature = "gbd")]
use tracing_subscriber::layer::Layered;
#[cfg(feature = "gbd")]
use tracing_subscriber::{reload, EnvFilter, Registry};

use crate::app::{App, Settings};
#[cfg(feature = "debug")]
use crate::app::Debug;
use crate::cli::Args;
#[cfg(feature = "doctor")]
use crate::doc::Doctor;
#[cfg(feature = "gbd")]
use crate::gbd::Debugger;
#[cfg(feature = "view")]
use crate::view::View;

mod app;
mod cli;
#[cfg(feature = "doctor")]
mod doc;
#[cfg(feature = "gbd")]
mod gbd;
mod pal;
#[cfg(feature = "view")]
mod view;

#[cfg(feature = "gbd")]
type Handle = reload::Handle<EnvFilter, Layered<Layer<Registry>, Registry>>;

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

fn main() -> Result<()> {
    // Install panic and error report handlers
    color_eyre::install()?;
    // Parse args
    let args = Args::parse();
    // Initialize logger
    let log = tracing_subscriber::fmt()
        .with_env_filter(&args.log)
        .with_filter_reloading();
    #[cfg(feature = "gbd")]
    let handle = log.reload_handle();
    log.init();

    // Read the boot ROM
    let boot = boot(args.boot)?;
    // Read the cartridge
    let cart = cart(args.rom, args.chk, args.force)?;
    let title = match cart.header().title.replace('\0', " ").trim() {
        "" => "Untitled",
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
    let win = Window::new(&title, SCREEN.width, SCREEN.height, opts)?;

    // Create debug views
    #[cfg(feature = "view")]
    let view = if args.view {
        Some(View::new(opts))
    } else {
        None
    };

    #[cfg(feature = "doctor")]
    // Open doctor logfile
    let doc = if let Some(path) = args.doc {
        Some(&path)
            .map(File::create)
            .transpose()
            .with_context(|| format!("failed to open doctor logfile: `{}`", path.display()))?
            .map(Doctor::new)
    } else {
        None
    };

    // Declare debugger
    #[cfg(feature = "gbd")]
    let gbd = args
        .gbd
        .then_some(Debugger::new())
        .map(|gdb| gdb.reload(handle));

    // Prepare settings
    let Args { pal, speed, .. } = args;
    let cfg = Settings { pal, speed };
    // Prepare debug info
    #[cfg(feature = "debug")]
    let debug = Debug {
        #[cfg(feature = "doctor")]
        doc,
        #[cfg(feature = "gbd")]
        gbd,
        #[cfg(feature = "view")]
        view,
    };
    // Prepare application
    let app = App {
        title,
        cfg,
        emu,
        win,
        #[cfg(feature = "debug")]
        debug,
    };

    // Run the app
    app.run()
}

fn boot(path: Option<PathBuf>) -> Result<Option<Boot>> {
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
    let boot = boot.as_ref().map(Boot::from);

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

        // Initialize the cartridge
        let cart = if force {
            // Force cartridge creation from ROM
            Cartridge::unchecked(&rom)
        } else if chk {
            // Check ROM integrity and create a cartridge
            let cart = Cartridge::checked(&rom)
                // exit on failure
                .with_context(|| format!("failed ROM integrity check: `{}`", path.display()))?;
            info!("Passed ROM integrity check");

            cart
        } else {
            // Attempt to create a cartridge
            Cartridge::new(&rom)
                // exit on failure
                .with_context(|| format!("failed to load cartridge: `{}`", path.display()))?
        };
        info!("Loaded cartridge:\n{}", cart.header());

        Ok(cart)
    } else {
        ensure!(force, "missing cartridge");
        warn!("Missing cartridge; defaulting to blank");
        Ok(Cartridge::blank())
    }
}
