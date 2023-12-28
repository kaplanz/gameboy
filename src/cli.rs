use std::path::PathBuf;

use clap::{Parser, ValueHint};

use crate::cfg::{Speed, Theme};

/// Game Boy emulator written in Rust.
#[allow(clippy::struct_excessive_bools)]
#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    /// Cartridge ROM image file.
    #[arg(required_unless_present("force"))]
    #[arg(value_hint = ValueHint::FilePath)]
    pub rom: Option<PathBuf>,

    /// Boot ROM image file.
    #[arg(short, long)]
    #[arg(value_hint = ValueHint::FilePath)]
    pub boot: Option<PathBuf>,

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

    /// Run without starting a UI.
    ///
    /// Runs the emulator without initializing and opening the UI. When compiled
    /// with GBD support, the emulator will also present a debugging prompt
    /// (implies --gbd).
    #[arg(short = 'H', long)]
    pub headless: bool,

    /// Logging level.
    ///
    /// A comma-separated list of logging directives, parsed using `env_logger`.
    /// Note that these filters are parsed after `RUST_LOG`.
    #[arg(short, long)]
    #[arg(default_value = "info")]
    #[arg(env = "RUST_LOG")]
    pub log: String,

    /// Exit after loading cartridge.
    ///
    /// Instead of entering the main emulation loop, return immediately after
    /// loading the cartridge ROM. This option could be used along with
    /// `--check` to validate a ROM, or using logging to print the cartridge
    /// header without actually performing any emulation.
    #[arg(short = 'x', long)]
    pub exit: bool,

    /// DMG-01 color palette.
    ///
    /// Defines the 2-bit color palette for the DMG-01 Game Boy model. The
    /// palette must be specified as a list of hex color values from lightest to
    /// darkest.
    #[arg(long = "palette")]
    #[arg(default_value_t)]
    #[arg(value_enum)]
    pub pal: Theme,

    /// Run at full-speed.
    ///
    /// Causes the emulator to run at the maximum possible speed the host
    /// machine supports.
    #[arg(short, long)]
    #[arg(default_value_t)]
    #[arg(value_enum)]
    pub speed: Speed,

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
    /// Starts emulation with the the Game Boy Debugger (GBD) prompt enabled.
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
