#![warn(clippy::pedantic)]

use std::fs::{self, File};
use std::io::{self, Read};
use std::net::UdpSocket;
use std::path::PathBuf;
use std::string::ToString;

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
use crate::dbg::doc::Doctor;
#[cfg(feature = "gbd")]
use crate::dbg::gbd::Readline;
#[cfg(feature = "view")]
use crate::gui::view::View;
use crate::gui::{Gui, Window};

mod app;
mod cfg;
mod cli;
#[cfg(feature = "debug")]
mod dbg;
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
                advise::error!("{err}");
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
            view: args
                .dbg
                .view
                .then(|| View::new().context("failed to initialize debug view"))
                .transpose()?,
        })
    };

    // Open serial link socket
    let link = args
        .hw
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
        .transpose()?;

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

    // Construct debugger
    #[cfg(feature = "gbd")]
    let gbd = args
        .dbg
        .gbd
        .then(|| -> Result<_> {
            // Construct a new `Debugger`
            let mut gbd = Debugger::new();
            // Initialize the prompt handle
            gbd.prompt(Box::new(
                Readline::new().context("failed to initialize readline")?,
            ));
            // Initialize the logger handle
            gbd.logger({
                Portal {
                    get: {
                        let handle = handle.clone();
                        Box::new(move || handle.with_current(ToString::to_string).unwrap())
                    },
                    set: {
                        let handle = handle.clone();
                        Box::new(move |filter: String| handle.reload(filter).unwrap())
                    },
                }
            });
            // Return the constructed debugger
            Ok(gbd)
        })
        .transpose()?;

    // Construct app options
    let cfg = app::Options {
        pal: args.gui.pal.unwrap_or(conf.gui.pal).into(),
        spd: args.gui.spd.unwrap_or(conf.gui.spd).freq(),
    };
    // Construct debug options
    #[cfg(feature = "debug")]
    let debug = app::Debug {
        #[cfg(feature = "doctor")]
        doc,
        #[cfg(feature = "gbd")]
        gbd,
    };
    // Construct peripherals
    let dev = app::Devices { link };
    // Construct application
    let app = App {
        title,
        cfg,
        #[cfg(feature = "debug")]
        dbg: debug,
        dev,
        emu,
        gui,
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
