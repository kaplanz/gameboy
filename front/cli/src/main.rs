#![warn(clippy::pedantic)]

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::PathBuf;

use clap::Parser;
use eyre::{ensure, Result, WrapErr};
use gameboy::core::dmg::cart::Cartridge;
use gameboy::core::dmg::{Boot, Dimensions, GameBoy, SCREEN};
#[cfg(feature = "gbd")]
use gameboy::gbd::{Debugger, Portal};
use log::{info, trace, warn};
use sysexits::ExitCode;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::app::App;
use crate::cfg::Config;
use crate::cli::Cli;
#[cfg(feature = "doctor")]
use crate::doc::Doctor;
#[cfg(feature = "view")]
use crate::gui::view::View;
use crate::gui::{Gui, Window};

mod app;
mod cfg;
mod cli;
#[cfg(feature = "doctor")]
mod doc;
mod gui;

/// Game Boy main clock frequency, set to 4,194,304 Hz.
const FREQUENCY: u32 = 4_194_304;

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    // Install panic and error report handlers
    color_eyre::install()?;
    // Parse args
    let args = Cli::parse();
    // Parse conf
    let conf: Config = {
        // Read file
        let path = &args.conf;
        match fs::read_to_string(path) {
            Ok(read) => Ok(read),
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => Ok(String::default()),
                _ => Err(err).with_context(|| format!("could not read: `{}`", path.display())),
            },
        }
        // Parse conf
        .map(|read| match toml::from_str(&read) {
            Ok(conf) => conf,
            Err(err) => {
                tell::error!("{err}");
                ExitCode::Config.exit();
            }
        })
    }
    .context("unable to load config")?;
    // Initialize logger
    let log = tracing_subscriber::fmt()
        .with_env_filter({
            let filter = args.log.as_deref().unwrap_or_default();
            EnvFilter::builder()
                .with_default_directive(LevelFilter::WARN.into())
                .parse(filter)
                .with_context(|| format!("failed to parse: \"{filter}\""))?
        })
        .with_filter_reloading();
    #[cfg(feature = "gbd")]
    let handle = log.reload_handle();
    log.init();
    // Log previous steps
    trace!("args: {args:#?}");
    trace!("conf: {conf:#?}");

    // Read the boot ROM
    let boot = boot(args.hw.boot.or(conf.hw.boot))?;
    // Read the cartridge
    let cart = cart(args.cart.rom, args.cart.chk, args.cart.force)?;
    let title = match cart.header().title.replace('\0', " ").trim() {
        "" => "Untitled",
        title => title,
    } // extract title from cartridge
    .to_string();

    // Exit early on `--exit`
    if args.exit {
        return Ok(());
    }

    // Create emulator instance
    let mut emu = if let Some(boot) = boot {
        GameBoy::with(boot)
    } else {
        GameBoy::new()
    };
    // Load the cartridge into the emulator
    emu.load(cart);

    // Initialize UI
    let gui = if args.gui.headless {
        None
    } else {
        let Dimensions { width, height } = SCREEN;
        Some(Gui {
            main: Window::new(&title, width, height)?,
            #[cfg(feature = "view")]
            view: args.dbg.view.then_some(View::new()?),
        })
    };

    #[cfg(feature = "doctor")]
    // Open doctor logfile
    let doc = args
        .dbg
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
    let gbd = args.dbg.gbd.then_some(Debugger::new()).map(|mut gdb| {
        gdb.logger({
            Portal {
                get: {
                    let handle = handle.clone();
                    Box::new(move || {
                        handle
                            .with_current(std::string::ToString::to_string)
                            .unwrap()
                    })
                },
                set: Box::new(move |filter: String| handle.reload(filter).unwrap()),
            }
        });
        gdb
    });

    // Construct app options
    let cfg = app::Options {
        pal: args.gui.pal.unwrap_or(conf.gui.pal).into(),
        spd: args.gui.speed.unwrap_or(conf.gui.speed).into(),
    };
    // Construct debug options
    #[cfg(feature = "debug")]
    let debug = app::Debug {
        #[cfg(feature = "doctor")]
        doc,
        #[cfg(feature = "gbd")]
        gbd,
    };
    // Construct application
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
