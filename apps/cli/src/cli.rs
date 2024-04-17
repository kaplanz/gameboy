//! Command-line interface.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Parser, ValueHint};

use crate::cfg::{self, Config};
use crate::def::NAME;

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
    #[clap(long)]
    #[clap(value_name = "PATH")]
    #[clap(default_value_os_t = cfg::path())]
    #[clap(value_hint = ValueHint::FilePath)]
    pub conf: PathBuf,

    /// Configuration data.
    #[clap(flatten)]
    #[clap(next_help_heading = None)]
    pub cfg: Config,

    /// Logging level.
    ///
    /// A comma-separated list of logging directives.
    #[clap(short, long, env = "RUST_LOG")]
    #[clap(value_name = "FILTER")]
    #[clap(help_heading = None)]
    pub log: Option<String>,

    /// Exit after loading cartridge.
    ///
    /// Instead of entering the main emulation loop, return immediately after
    /// loading the cartridge ROM. This option could be used along with
    /// `--check` to validate a ROM, or, if logging is enabled, to print the
    /// cartridge header without emulating.
    #[clap(short = 'x', long)]
    #[clap(help_heading = "Interface")]
    pub exit: bool,

    /// Run without UI.
    ///
    /// Starts the emulator without initializing and opening the UI.
    #[clap(short = 'H', long)]
    #[clap(help_heading = "Interface")]
    pub headless: bool,

    /// Serial connection.
    #[clap(flatten)]
    #[clap(next_help_heading = "Interface")]
    pub link: Option<Link>,

    /// Debugging options.
    #[clap(flatten)]
    #[clap(next_help_heading = "Debug")]
    pub dbg: Debug,
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
    /// Opens a UDP socket for serial communications at the specified address.
    #[clap(long)]
    #[clap(value_name = "ADDR")]
    #[clap(required = false, requires = "host")]
    pub peer: SocketAddr,
}

/// Debugging options.
#[derive(Args, Debug)]
pub struct Debug {
    #[cfg(feature = "doc")]
    /// Doctor logfile path.
    ///
    /// Enables logging at the provided path of the emulator's state after every
    /// instruction in the format used by Gameboy Doctor.
    #[clap(long)]
    #[clap(value_name = "PATH")]
    #[clap(value_hint = ValueHint::FilePath)]
    pub doc: Option<PathBuf>,

    #[cfg(feature = "gbd")]
    /// Enable interactive Game Boy Debugger.
    ///
    /// Starts emulation with the Game Boy Debugger (GBD) enabled, prompting
    /// after.
    #[clap(short = 'i', long)]
    pub gbd: bool,

    #[cfg(feature = "win")]
    /// Open debug windows.
    ///
    /// Causes the emulator to open the debug windows, providing graphical
    /// rendering of video RAM contents.
    #[clap(long)]
    pub win: bool,
}
