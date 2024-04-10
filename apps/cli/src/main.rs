#![warn(clippy::pedantic)]

use std::fs::File;
use std::io::Read;
use std::net::UdpSocket;
use std::ops::Not;
use std::path::Path;
use std::string::ToString;

use anyhow::{ensure, Context, Result};
use clap::Parser;
use log::{error, info, trace, warn};
use rugby::core::dmg::cart::Cartridge;
use rugby::core::dmg::{Boot, Dimensions, GameBoy, SCREEN};
#[cfg(feature = "gbd")]
use rugby::gbd::Debugger;
use rugby::gbd::Portal;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::app::App;
use crate::cfg::Config;
use crate::cli::Cli;
#[cfg(feature = "doctor")]
use crate::dbg::doc::Doctor;
#[cfg(feature = "gbd")]
use crate::dbg::gbd::Console;
use crate::err::Exit;
#[cfg(feature = "view")]
use crate::gui::view::View;
use crate::gui::{Gui, Window};

mod app;
mod cfg;
mod cli;
#[cfg(feature = "debug")]
mod dbg;
mod def;
mod dir;
mod err;
mod gui;

/// Name of this crate.
///
/// This may be used for base subdirectories.
pub const NAME: &str = "rugby";

/// Game Boy main clock frequency, set to 4,194,304 Hz.
pub const FREQ: u32 = 4_194_304;

/// Temporary substitute for `try` trait to perform `?` desugaring.
macro_rules! check {
    ($res:expr) => {{
        let res = $res;
        match res {
            Ok(okay) => okay,
            Err(err) => return err.into(),
        }
    }};
}

#[allow(clippy::too_many_lines)]
fn main() -> Exit {
    // Parse args
    let mut args = Cli::parse();
    // Load config
    args.cfg.merge(check!(
        Config::load(&args.conf).context("could not load configuration")
    ));
    // Initialize logger
    #[allow(unused_variables)]
    let log = check!(
        logger(args.log.as_deref().unwrap_or_default()).context("could not initialize logger")
    );
    // Log previous steps
    trace!("{args:#?}");

    // Prepare application
    let app = {
        // Load ROMs
        let boot = check!(args
            .cfg
            .hw
            .boot
            .as_deref()
            .map(boot)
            .transpose()
            .context("could not load boot ROM"));
        let cart = check!(cart(
            args.cfg.sw.rom.as_deref(),
            args.cfg.sw.check,
            args.cfg.sw.force
        )
        .context("could not load cartridge"));

        // Exit early on `--exit`
        if args.exit {
            return Exit::Success;
        }

        // Initialize graphics
        let title = cart.title().to_string();
        let gui = check!(args
            .headless
            .not()
            .then(|| -> Result<_> {
                let Dimensions { width, height } = SCREEN;
                Ok(Gui {
                    main: Window::new(&title, width, height)?,
                    #[cfg(feature = "view")]
                    view: args
                        .dbg
                        .view
                        .then(|| View::new().context("could not initialize debug view"))
                        .transpose()?,
                })
            })
            .transpose());

        // Instantiate emulator
        let mut emu = boot.map_or_else(GameBoy::new, GameBoy::with);
        emu.load(cart); // load cartridge into emulator

        // Open link cable
        let link = check!(args
            .link
            .map(|cli::Link { host, peer }| -> Result<_> {
                // Bind host to local address
                let sock = UdpSocket::bind(host)
                    .with_context(|| format!("failed to bind local socket: `{host}`"))?;
                // Connect to peer address
                sock.connect(peer)
                    .with_context(|| format!("failed to connect to peer: `{peer}`"))?;
                // Set socket options
                sock.set_nonblocking(true)
                    .context("failed to set non-blocking")?;
                // Return completed link
                Ok(sock)
            })
            .transpose()
            .context("could not open link cable"));

        // Open log file
        #[cfg(feature = "doctor")]
        let doc = check!(args
            .dbg
            .doc
            .map(|path| -> Result<_> {
                // Create logfile
                let file = File::create(&path)
                    .with_context(|| format!("failed to open: `{}`", path.display()))?;
                // Construct a doctor instance
                Ok(Doctor::new(file))
            })
            .transpose()
            .context("could not open log file"));

        // Prepare debugger
        #[cfg(feature = "gbd")]
        let gbd = check!(args
            .dbg
            .gbd
            .then(|| -> Result<_> {
                // Construct a new `Debugger`
                let mut gbd = Debugger::new();
                // Initialize prompt handle
                gbd.prompt(Box::new({
                    Console::new().context("failed to initialize readline")?
                }));
                // Initialize logger handle
                gbd.logger(log);
                // Return constructed debugger
                Ok(gbd)
            })
            .transpose()
            .context("could not prepare debugger"));

        // Configure application options
        let cfg = app::Options {
            title,
            pal: args.cfg.ui.pal.unwrap_or_default().into(),
            spd: args.cfg.ui.spd.unwrap_or_default().freq(),
        };
        // Configure application debug
        #[cfg(feature = "debug")]
        let dbg = app::Debug {
            #[cfg(feature = "doctor")]
            doc,
            #[cfg(feature = "gbd")]
            gbd,
        };
        // Construct application devices
        let dev = app::Devices { link };
        // Construct application
        App {
            cfg,
            #[cfg(feature = "debug")]
            dbg,
            dev,
            emu,
            gui,
        }
    };
    // Run application
    check!(app.run());

    // Terminate normally
    Exit::Success
}

fn logger(filter: &str) -> Result<Portal<String>> {
    // Construct logger
    let log = tracing_subscriber::fmt()
        .with_env_filter({
            EnvFilter::builder()
                .with_default_directive(LevelFilter::WARN.into())
                .parse(filter)
                .with_context(|| format!("failed to parse: {filter:?}"))?
        })
        .with_filter_reloading();
    // Extract handle
    let handle = {
        Portal {
            get: {
                let handle = log.reload_handle();
                Box::new(move || handle.with_current(ToString::to_string).unwrap())
            },
            set: {
                let handle = log.reload_handle();
                Box::new(move |filter: String| handle.reload(filter).unwrap())
            },
        }
    };
    // Install logger
    log.init();
    // Return handle
    Ok(handle)
}

/// Read and load a boot ROM instance from a file.
fn boot(path: &Path) -> Result<Boot> {
    // Read ROM file
    let boot = {
        // Open ROM file
        let mut file =
            File::open(path).with_context(|| format!("failed to open: `{}`", path.display()))?;
        // Read ROM into a buffer (must be exactly 256 bytes)
        let mut buf = [0u8; 0x0100];
        file.read_exact(&mut buf)
            .with_context(|| format!("failed to read: `{}`", path.display()))?;
        info!(
            "read {} bytes from boot ROM: `{}`",
            buf.len(),
            path.display(),
        );
        // Return ROM contents
        buf
    };
    // Initialize boot ROM
    let boot = Boot::from(&boot);

    Ok(boot)
}

/// Read and load a cartridge instance from a file.
fn cart(path: Option<&Path>, check: bool, force: bool) -> Result<Cartridge> {
    if let Some(path) = path {
        // Read ROM file
        let rom = {
            // Open ROM file
            let file = File::open(path)
                .with_context(|| format!("failed to open: `{}`", path.display()))?;
            // Read ROM into a buffer
            let mut buf = Vec::new();
            let nbytes = file
                // Game Paks manufactured by Nintendo have a maximum 8 MiB ROM
                .take(0x0080_0000)
                .read_to_end(&mut buf)
                .with_context(|| format!("failed to read: `{}`", path.display()))?;
            info!("read {nbytes} bytes from ROM: `{}`", path.display());
            // Return ROM contents
            buf
        };

        // Initialize cartridge
        let cart = if force {
            // Force cartridge creation from ROM
            Cartridge::checked(&rom)
                .inspect_err(|err| error!("{err:#}"))
                .ok() // discard the error
                .unwrap_or_else(|| Cartridge::unchecked(&rom))
        } else if check {
            // Check ROM integrity and create a cartridge
            Cartridge::checked(&rom)
                .inspect(|_| info!("passed ROM integrity check"))
                .with_context(|| format!("failed to load: `{}`", path.display()))?
        } else {
            // Attempt to create a cartridge
            Cartridge::new(&rom).with_context(|| format!("failed to load: `{}`", path.display()))?
        };
        info!("loaded cartridge:\n{}", cart.header());

        Ok(cart)
    } else {
        // Initialize blank cartridge
        ensure!(force, "missing cartridge; did not specify `--force`");
        warn!("missing cartridge; defaulting to blank");
        Ok(Cartridge::blank())
    }
}
