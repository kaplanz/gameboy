#![warn(clippy::pedantic)]

use anyhow::Context;
use clap::Parser;
use log::{trace, warn};

use crate::cfg::Config;
use crate::cli::Cli;
use crate::err::{Exit, Result};

mod app;
mod cfg;
mod cli;
#[cfg(feature = "debug")]
mod dbg;
mod def;
mod dir;
mod err;

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
    args.cfg
        .merge(Config::load(&args.conf).context("could not load configuration")?);
    // Initialize logger
    #[allow(unused)]
    let log = setup::log(args.log.as_deref().unwrap_or_default())
        .context("could not initialize logger")?;
    // Log previous steps
    trace!("{args:#?}");

    // Prepare emulator
    let emu = setup::emu(&args)?;
    // Perform early exit
    if args.exit {
        return Ok(());
    }
    // Prepare application
    let app = setup::app(&args, emu, log)?;
    // Run application
    app.run()?;

    // Terminate normally
    Ok(())
}

mod setup {
    use std::fs::File;
    use std::io::Read;
    use std::net::UdpSocket;
    use std::ops::Not;
    use std::path::Path;
    use std::string::ToString;

    use anyhow::{ensure, Context, Result};
    use log::{debug, error, info, warn};
    use rugby::core::dmg::cart::Cartridge;
    use rugby::core::dmg::{Boot, GameBoy, LCD};
    use rugby::emu::cart::Support as _;
    use rugby::emu::video;
    #[cfg(feature = "gbd")]
    use rugby::gbd::Debugger;
    use rugby::gbd::Portal;
    use tracing_subscriber::filter::LevelFilter;
    use tracing_subscriber::EnvFilter;

    use crate::app::{self, App, Graphics};
    use crate::cli::{self, Cli};
    #[cfg(feature = "doc")]
    use crate::dbg::doc::Doctor;
    #[cfg(feature = "gbd")]
    use crate::dbg::gbd::Console;
    use crate::def::NAME;

    type Log = Portal<String>;

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

    pub fn emu(args: &Cli) -> Result<GameBoy> {
        // Load boot ROM
        let boot = args
            .cfg
            .hw
            .boot
            .as_deref()
            .map(boot)
            .transpose()
            .context("could not load boot ROM")?;
        // Load cartridge
        let load = |path| cart(path, args.cfg.sw.check, args.cfg.sw.force);
        let cart = args
            .cfg
            .sw
            .rom
            .as_deref()
            .map(load)
            .transpose()
            .context("could not load cartridge")?;

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
        let boot = Boot::from(&rom);
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

        // Return success
        Ok(cart)
    }

    #[allow(unused)]
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

    fn link(cli::Link { host, peer }: &cli::Link) -> Result<UdpSocket> {
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

    #[cfg(feature = "doc")]
    fn doc(path: &Path) -> Result<Doctor> {
        // Create logfile
        let file =
            File::create(path).with_context(|| format!("failed to open: `{}`", path.display()))?;
        // Construct a doctor instance
        Ok(Doctor::new(file))
    }

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
}
