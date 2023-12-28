#![warn(clippy::pedantic)]

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::Parser;
use eyre::{ensure, Result, WrapErr};
use gameboy::core::dmg::Dimensions;
use gameboy::dmg::cart::Cartridge;
use gameboy::dmg::{Boot, GameBoy, SCREEN};
use log::{info, warn};
#[cfg(feature = "gbd")]
use tracing_subscriber::fmt::Layer;
#[cfg(feature = "gbd")]
use tracing_subscriber::layer::Layered;
#[cfg(feature = "gbd")]
use tracing_subscriber::{reload, EnvFilter, Registry};

use crate::app::App;
#[cfg(feature = "debug")]
use crate::app::Debug;
use crate::cfg::Settings;
use crate::cli::Args;
#[cfg(feature = "doctor")]
use crate::doc::Doctor;
#[cfg(feature = "gbd")]
use crate::gbd::Debugger;
#[cfg(feature = "view")]
use crate::gui::view::View;
use crate::gui::{Gui, Window};

mod app;
mod cfg;
mod cli;
#[cfg(feature = "doctor")]
mod doc;
#[cfg(feature = "gbd")]
mod gbd;
mod gui;
mod pal;

#[cfg(feature = "gbd")]
type Handle = reload::Handle<EnvFilter, Layered<Layer<Registry>, Registry>>;

/// Game Boy main clock frequency, set to 4,194,304 Hz.
const FREQ: u32 = 0x0040_0000;

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

    // Initialize UI
    let gui = if args.headless {
        None
    } else {
        let Dimensions { width, height } = SCREEN;
        Some(Gui {
            main: Window::new(&title, width, height)?,
            #[cfg(feature = "view")]
            view: args.view.then_some(View::new()?),
        })
    };

    #[cfg(feature = "doctor")]
    // Open doctor logfile
    let doc = args
        .doc
        .map(|path| -> Result<_> {
            // Create logfile
            let f = File::create(&path)
                .with_context(|| format!("failed to open doctor logfile: `{}`", path.display()))?;
            // Construct a doctor instance
            Ok(Doctor::new(f))
        })
        .transpose()?;

    // Declare debugger
    #[cfg(feature = "gbd")]
    let gbd = args
        .gbd
        .then_some(Debugger::new())
        .map(|gdb| gdb.set_log(handle));

    // Prepare settings
    let Args { pal, speed, .. } = args;
    let cfg = Settings {
        pal: pal.into(),
        spd: speed.into(),
    };
    // Prepare debug info
    #[cfg(feature = "debug")]
    let debug = Debug {
        #[cfg(feature = "doctor")]
        doc,
        #[cfg(feature = "gbd")]
        gbd,
    };
    // Prepare application
    let app = App {
        title,
        cfg,
        emu,
        gui,
        #[cfg(feature = "debug")]
        dbg: debug,
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
            // Read boot ROM into a buffer (must be exactly 256 bytes)
            let mut buf = [0u8; 0x0100];
            f.read_exact(&mut buf)
                .with_context(|| format!("failed to read full boot ROM: `{}`", boot.display()))?;
            info!(
                "read {} bytes from boot ROM: `{}`",
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
            let mut buf = Vec::new();
            let nbytes = f
                // Game Paks manufactured by Nintendo have a maximum 8 MiB ROM
                .take(0x0080_0000)
                .read_to_end(&mut buf)
                .with_context(|| format!("failed to read ROM: `{}`", path.display()))?;
            info!("read {nbytes} bytes from ROM: `{}`", path.display());

            buf
        };

        // Initialize the cartridge
        let cart = if force {
            // Force cartridge creation from ROM
            Cartridge::unchecked(&rom)
        } else if chk {
            // Check ROM integrity and create a cartridge
            let cart = Cartridge::checked(&rom)
                .with_context(|| format!("failed ROM integrity check: `{}`", path.display()))?;
            info!("passed ROM integrity check");

            cart
        } else {
            // Attempt to create a cartridge
            Cartridge::new(&rom)
                .with_context(|| format!("failed to load cartridge: `{}`", path.display()))?
        };
        info!("loaded cartridge:\n{}", cart.header());

        Ok(cart)
    } else {
        ensure!(force, "missing cartridge");
        warn!("missing cartridge; defaulting to blank");
        Ok(Cartridge::blank())
    }
}
