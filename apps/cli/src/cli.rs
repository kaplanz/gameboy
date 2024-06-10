//! Command-line interface.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Parser, ValueHint};
use rugby_cfg::Config;

use crate::NAME;

/// Emulate the Nintendo Game Boy.
///
/// Cycle-accurate emulation with support for custom palettes, configurable
/// speed, interactive debugging, and more!
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Parser)]
#[clap(name = NAME, author, version, about, long_about)]
pub struct Cli {
    /// Configuration file.
    ///
    /// When options are specified in multiple locations, they will be applied
    /// with the following precedence: cli > env > file.
    #[clap(long, env = rugby_cfg::env::CFG)]
    #[clap(value_name = "PATH")]
    #[clap(default_value_os_t = crate::cfg::path())]
    #[clap(hide_default_value = std::env::var(rugby_cfg::env::CFG).is_ok())]
    #[clap(hide_env_values    = std::env::var(rugby_cfg::env::CFG).is_err())]
    #[clap(value_hint = ValueHint::FilePath)]
    pub conf: PathBuf,

    /// Runtime options.
    #[clap(flatten)]
    #[clap(next_help_heading = "Runtime")]
    pub run: Runtime,

    /// Configuration data.
    #[clap(flatten)]
    #[clap(next_help_heading = None)]
    pub cfg: Config,

    /// Serial connection.
    #[clap(flatten)]
    #[clap(next_help_heading = "Serial")]
    pub link: Option<Link>,

    /// Debugging options.
    #[clap(flatten)]
    #[clap(next_help_heading = "Debug")]
    pub dbg: Debug,
}

/// Runtime options.
#[derive(Args, Debug)]
pub struct Runtime {
    /// Exit without running.
    ///
    /// Instead of entering the main emulation loop, return immediately after
    /// loading the cartridge ROM. This option could be used along with
    /// `--check` to validate a ROM, or, if logging is enabled, to print the
    /// cartridge header without emulating.
    #[clap(short = 'x', long)]
    pub exit: bool,

    /// Run in headless mode.
    ///
    /// Starts without initializing or opening the UI.
    #[clap(short = 'H', long)]
    pub headless: bool,
}

/// Serial connection.
#[derive(Args, Debug)]
pub struct Link {
    /// Link cable local address.
    ///
    /// Binds a local UDP socket to the specified address for serial
    /// communications.
    #[clap(long)]
    #[clap(value_name = "ADDR")]
    #[clap(required = false, requires = "peer")]
    pub host: SocketAddr,

    /// Link cable peer address.
    ///
    /// Opens a UDP socket at the specified address for serial communications.
    #[clap(long)]
    #[clap(value_name = "ADDR")]
    #[clap(required = false, requires = "host")]
    pub peer: SocketAddr,
}

/// Debugging options.
#[derive(Args, Debug)]
pub struct Debug {
    /// Doctor logfile path.
    ///
    /// Enables logging at the provided path of the emulator's state after every
    /// instruction in the format used by Gameboy Doctor.
    #[cfg(feature = "doc")]
    #[clap(long)]
    #[clap(value_name = "PATH")]
    #[clap(value_hint = ValueHint::FilePath)]
    pub doc: Option<PathBuf>,

    /// Enable interactive debugging.
    ///
    /// Starts with Game Boy Debugger (GBD) enabled, presenting the prompt after
    /// loading.
    #[cfg(feature = "gbd")]
    #[clap(short = 'i', long)]
    pub gbd: bool,

    /// Enable VRAM debug windows.
    ///
    /// Starts with debug windows opened, visually rendering VRAM contents.
    #[cfg(feature = "win")]
    #[clap(long)]
    pub win: bool,
}
