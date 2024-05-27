#![warn(clippy::pedantic)]

use std::path::Path;

use anyhow::Context;
use clap::Parser;
use log::{trace, warn};
use rugby::prelude::*;

use crate::cfg::Config;
use crate::cli::Cli;
use crate::err::{Exit, Result};

mod app;
mod cfg;
mod cli;
#[cfg(feature = "debug")]
mod dbg;
mod dir;
mod err;
mod opt;

fn main() -> Exit {
    match run() {
        Ok(()) => Exit::Success,
        Err(e) => Exit::Failure(e),
    }
}

fn run() -> Result<()> {
    // Parse args
    let mut args = Cli::parse();
    // Load config
    args.cfg.merge({
        // Parse config from file
        let mut cfg = Config::load(&args.conf).context("could not load configuration")?;
        // Rebase paths to parent
        cfg.rebase(args.conf.parent().unwrap_or(Path::new("")));
        // Merge with args
        cfg
    });
    // Initialize logger
    #[cfg_attr(not(feature = "gbd"), allow(unused, clippy::let_unit_value))]
    let log = build::log(args.log.as_deref().unwrap_or_default())
        .context("could not initialize logger")?;
    // Log previous steps
    trace!("{args:#?}");

    // Prepare emulator
    let emu = build::emu(&args)?;
    // Perform early exit
    if args.exit {
        return Ok(());
    }
    // Prepare application
    let mut app = build::app(&args, emu, log)?;
    // Flash cartridge RAM
    build::flash(
        args.cfg.sw.ram().as_deref(),
        app.emu.cart_mut(),
        args.cfg.sw.save.unwrap_or_default(),
    )
    .context("could not flash RAM")?;
    // Run application
    app.run()?;
    // Dump cartridge RAM
    build::dump(
        app.emu.cart(),
        args.cfg.sw.ram().as_deref(),
        args.cfg.sw.save.unwrap_or_default(),
    )
    .context("could not dump RAM")?;

    // Terminate normally
    Ok(())
}

/// Build helpers.
mod build {
    use std::fs::File;
    use std::io::Read;
    use std::net::UdpSocket;
    use std::ops::Not;
    use std::path::Path;

    use anyhow::{anyhow, ensure, Context, Result};
    use log::{debug, error, info, warn};
    use rugby::core::dmg::cart::mbc::Mbc;
    use rugby::core::dmg::{Boot, Cartridge, GameBoy, LCD};
    use rugby::emu::video;
    use rugby::prelude::*;
    #[cfg(feature = "gbd")]
    use rugby_gbd::{Debugger, Portal};
    use tracing_subscriber::filter::LevelFilter;
    use tracing_subscriber::EnvFilter;

    use crate::app::gui::Cable;
    use crate::app::{self, App, Graphics};
    use crate::cli::{self, Cli};
    #[cfg(feature = "doc")]
    use crate::dbg::doc::Doctor;
    #[cfg(feature = "gbd")]
    use crate::dbg::gbd::Console;
    use crate::opt::{Tristate, NAME};

    #[cfg(feature = "gbd")]
    type Log = Portal<String>;
    #[cfg(not(feature = "gbd"))]
    type Log = ();

    /// Installs the global logger, returning an abstracted reload handle.
    pub fn log(filter: &str) -> Result<Log> {
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
        #[cfg(feature = "gbd")]
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
        #[cfg(not(feature = "gbd"))]
        let handle = ();
        // Install logger
        log.init();
        // Return handle
        Ok(handle)
    }

    /// Builds an emulator instance.
    pub fn emu(args: &Cli) -> Result<GameBoy> {
        // Load cartridge
        let cart = {
            let cart = &args.cfg.sw;
            cart.rom
                .as_deref()
                .map(|path| self::cart(path, cart.check, cart.force))
                .transpose()
        }
        .context("could not load cartridge")?;
        // Load boot ROM
        let boot = (|| {
            let boot = &args.cfg.hw.boot;
            boot.skip
                .not()
                .then(|| boot.image.as_deref().map(self::boot))
                .map(|res| res.ok_or(anyhow!("missing path; unspecified by `--boot`")))
                .transpose()?
                .transpose()
        })() // NOTE: This weird syntax is in lieu of using unstable try blocks.
        .context("could not load boot ROM")?;

        // Instantiate emulator
        let mut emu = boot.map_or_else(GameBoy::new, GameBoy::with);
        // Insert cartridge
        if let Some(cart) = cart {
            emu.load(cart);
        } else {
            // Handle missing cartridge
            ensure!(
                args.cfg.sw.force,
                "missing cartridge; did not specify `--force`"
            );
            warn!("missing cartridge");
        }

        // Return emulator
        Ok(emu)
    }

    /// Read and load a boot ROM instance from a file.
    fn boot(path: &Path) -> Result<Boot> {
        // Read ROM file
        let rom = {
            // Open ROM file
            let mut file = File::open(path)
                .with_context(|| format!("failed to open: `{}`", path.display()))?;
            // Read ROM into a buffer (must be exactly 256 bytes)
            let mut buf = [0u8; 0x0100];
            file.read_exact(&mut buf)
                .with_context(|| format!("failed to read: `{}`", path.display()))?;
            let nbytes = buf.len();
            debug!("read {nbytes} bytes: `{}`", path.display());
            // Use ROM contents
            buf
        };

        // Initialize boot ROM
        let boot = Boot::from(rom);
        info!("loaded boot ROM");

        // Return success
        Ok(boot)
    }

    /// Read and load a cartridge instance from a file.
    fn cart(path: &Path, check: bool, force: bool) -> Result<Cartridge> {
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
            debug!("read {nbytes} bytes: `{}`", path.display());
            // Use ROM contents
            buf
        };

        // Initialize cartridge
        let cart = if force {
            // If both force and check are supplied, default to force
            if check {
                warn!("use of `--force` overrides `--check`");
            }
            // Force cartridge construction
            Cartridge::unchecked
        } else if check {
            // Check cartridge integrity
            Cartridge::checked
        } else {
            // Construct a cartridge
            Cartridge::new
        }(&rom)
        .with_context(|| format!("failed to load: `{}`", path.display()))?;
        info!("loaded cartridge:\n{}", cart.header());

        // Return success
        Ok(cart)
    }

    /// Builds an application instance.
    #[allow(unused, clippy::needless_pass_by_value)]
    pub fn app(args: &Cli, emu: GameBoy, log: Log) -> Result<App> {
        // Initialize graphics
        let gui = args
            .headless
            .not()
            .then(|| {
                gui(
                    #[cfg(feature = "win")]
                    args.dbg.win,
                )
            })
            .transpose()
            .context("could not open graphics")?;

        // Open link cable
        let lnk = args
            .link
            .as_ref()
            .map(link)
            .transpose()
            .context("could not open link cable")?;

        // Open log file
        #[cfg(feature = "doc")]
        let doc = args
            .dbg
            .doc
            .as_deref()
            .map(doc)
            .transpose()
            .context("could not open log file")?;

        // Prepare debugger
        #[cfg(feature = "gbd")]
        let gbd = args
            .dbg
            .gbd
            .then(|| gbd(log))
            .transpose()
            .context("could not prepare debugger")?;

        // Construct application
        let app = App {
            cfg: app::Options {
                spd: args.cfg.ui.spd.clone().unwrap_or_default().freq(),
            },
            ctx: None,
            #[cfg(feature = "debug")]
            dbg: app::Debug {
                #[cfg(feature = "doc")]
                doc,
                #[cfg(feature = "gbd")]
                gbd,
                #[cfg(feature = "win")]
                win: args.dbg.win,
            },
            emu,
            gui: app::Frontend {
                cfg: app::gui::Options {
                    pal: args.cfg.ui.pal.clone().unwrap_or_default().into(),
                },
                win: gui,
                lnk,
            },
        };

        // Return app
        Ok(app)
    }

    /// Builds a graphics instance.
    fn gui(#[cfg(feature = "win")] dbg: bool) -> Result<Graphics> {
        // Calculate aspect
        let video::Aspect { wd, ht } = LCD;
        let size = (wd.into(), ht.into()).into();
        // Construct GUI
        #[allow(unused)]
        let mut gui = Graphics::new(NAME, size).context("failed to open main window")?;
        // Open debug windows
        #[cfg(feature = "win")]
        if dbg {
            gui.dbg.all().context("failed to open debug windows")?;
        };
        // Return GUI
        Ok(gui)
    }

    /// Builds a link cable instance.
    fn link(cli::Link { host, peer }: &cli::Link) -> Result<Cable> {
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
    }

    /// Builds a doctor logfile instance.
    #[cfg(feature = "doc")]
    fn doc(path: &Path) -> Result<Doctor> {
        // Create logfile
        let file =
            File::create(path).with_context(|| format!("failed to open: `{}`", path.display()))?;
        // Construct a doctor instance
        Ok(Doctor::new(file))
    }

    /// Builds a debugger instance.
    #[cfg(feature = "gbd")]
    fn gbd(log: Log) -> Result<Debugger> {
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
    }

    /// Flashes the cartridge RAM from a save file.
    pub fn flash(path: Option<&Path>, cart: Option<&mut Cartridge>, save: Tristate) -> Result<()> {
        let Some(path) = path else {
            return Ok(());
        };
        let Some(cart) = cart else {
            return Ok(());
        };
        if save == Tristate::Never {
            return Ok(());
        }
        if save == Tristate::Auto && cart.header().info.has_battery() {
            return Ok(());
        }
        if !path.exists() {
            return Ok(());
        }

        // Open RAM file
        let mut file = File::open(path)
            .with_context(|| format!("failed to open: `{}`", path.display()))?
            .take(0x0002_0000); // cartridge ROM has a maximum of 128 KiB

        // Load into cartridge
        let nbytes = cart
            .body_mut()
            .flash(&mut file)
            .with_context(|| format!("failed to read: `{}", path.display()))?;
        info!("flashed {nbytes} bytes: `{}`", path.display());

        Ok(())
    }

    /// Dumps the cartridge RAM to a save file.
    pub fn dump(cart: Option<&Cartridge>, path: Option<&Path>, save: Tristate) -> Result<()> {
        let Some(cart) = cart else {
            return Ok(());
        };
        let Some(path) = path else {
            return Ok(());
        };
        if save == Tristate::Never {
            return Ok(());
        }
        if save == Tristate::Auto && !cart.header().info.has_battery() {
            return Ok(());
        }
        if !cart.header().info.has_ram() {
            error!("cannot dump: cartridge does not support RAM");
            return Ok(());
        }

        // Open RAM file
        let mut file =
            File::create(path).with_context(|| format!("failed to open: `{}`", path.display()))?;

        // Save from cartridge
        let nbytes = cart
            .body()
            .dump(&mut file)
            .with_context(|| format!("failed to write: `{}", path.display()))?;
        info!("dumped {nbytes} bytes: `{}`", path.display());

        Ok(())
    }
}
