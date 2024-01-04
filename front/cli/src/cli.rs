use std::path::PathBuf;

use clap::{Args, Parser, ValueHint};

use crate::cfg::{self, Model, Palette, Speed};

/// Game Boy emulator written in Rust.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// Emulator configuration file.
    ///
    /// Options specified in the configuration file have lower precedence to
    /// those specified in the environment or on the command line.
    #[arg(long)]
    #[arg(default_value_os_t = cfg::dir().join("config.toml"))]
    #[arg(value_hint = ValueHint::FilePath)]
    pub conf: PathBuf,

    /// Logging level.
    ///
    /// A comma-separated list of logging directives, parsed using `env_logger`.
    /// Note that these filters are parsed after `RUST_LOG`.
    #[arg(short, long)]
    #[arg(env = "RUST_LOG")]
    pub log: Option<String>,

    /// Exit after loading cartridge.
    ///
    /// Instead of entering the main emulation loop, return immediately after
    /// loading the cartridge ROM. This option could be used along with
    /// `--check` to validate a ROM, or, if logging is enabled, to print the
    /// cartridge header without emulating.
    #[arg(short = 'x', long)]
    pub exit: bool,

    /// Cartridge options.
    #[command(flatten)]
    pub cart: Cartridge,

    /// Hardware options.
    #[command(flatten)]
    pub hw: Hardware,

    /// User interface options.
    #[command(flatten)]
    pub gui: Interface,

    /// Debugging options.
    #[command(flatten)]
    pub dbg: Debug,
}

/// Cartridge options.
#[derive(Args, Debug)]
pub struct Cartridge {
    /// Cartridge ROM image file.
    ///
    /// A cartridge will be constructed from the data specified in the ROM. The
    /// cartridge header specifies precisely what hardware will be instantiated.
    #[arg(required_unless_present("force"))]
    #[arg(value_hint = ValueHint::FilePath)]
    pub rom: Option<PathBuf>,

    /// Check ROM integrity.
    ///
    /// Verifies that both the header and global checksums match the data within
    /// the ROM.
    #[arg(short, long = "check")]
    #[arg(conflicts_with("force"))]
    pub chk: bool,

    /// Force cartridge construction.
    ///
    /// Causes the cartridge generation to always succeed, even if the ROM does
    /// not contain valid data.
    #[arg(short, long)]
    pub force: bool,
}

/// Hardware options.
#[derive(Args, Debug, Default)]
pub struct Hardware {
    /// Boot ROM image file.
    ///
    /// Embedded firmware ROM executed upon booting.
    #[arg(short, long)]
    #[arg(value_hint = ValueHint::FilePath)]
    pub boot: Option<PathBuf>,

    /// Game Boy hardware model.
    ///
    /// Console and revision specification.
    #[arg(long)]
    #[arg(value_enum)]
    pub model: Option<Model>,
}

/// User interface options.
#[derive(Args, Debug, Default)]
pub struct Interface {
    /// Run without UI.
    ///
    /// Starts the emulator without initializing and opening the UI. When
    /// compiled with GBD support, the emulator will also present a debugging
    /// prompt (implies `--gbd`).
    #[arg(short = 'H', long)]
    pub headless: bool,

    /// DMG color palette.
    ///
    /// Defines the 2-bit color palette for the DMG-01 Game Boy model. The
    /// palette must be specified as a list of hex color values from lightest to
    /// darkest.
    #[arg(long = "palette")]
    #[arg(value_enum)]
    pub pal: Option<Palette>,

    /// Simulated clock speed.
    ///
    /// Causes the emulator to run at the maximum possible speed the host
    /// machine supports.
    #[arg(short, long)]
    #[arg(value_enum)]
    pub speed: Option<Speed>,
}

/// Debugging options.
#[derive(Args, Debug)]
pub struct Debug {
    #[cfg(feature = "doctor")]
    /// Doctor logfile path.
    ///
    /// Enables logging at the provided path of the emulator's state after every
    /// instruction in the format used by Gameboy Doctor.
    #[arg(long = "doctor")]
    #[arg(help_heading = "Debug")]
    #[arg(value_hint = ValueHint::FilePath)]
    pub doc: Option<PathBuf>,

    #[cfg(feature = "gbd")]
    /// Enable interactive Game Boy Debugger.
    ///
    /// Starts emulation with the Game Boy Debugger (GBD) enabled, prompting after .
    #[arg(short = 'i', long)]
    #[arg(help_heading = "Debug")]
    pub gbd: bool,

    #[cfg(feature = "view")]
    /// Open debug view windows.
    ///
    /// Causes the emulator to open the debug views, providing graphical
    /// rendering of video RAM contents.
    #[arg(long)]
    #[arg(help_heading = "Debug")]
    pub view: bool,
}
