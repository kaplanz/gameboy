//! Application configuration.

use std::path::Path;

pub use crate::val::{Palette, Speed};
use crate::Conf;

/// Application options.
#[derive(Debug, Default)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Application {
    /// Logging level.
    ///
    /// A comma-separated list of logging directives.
    #[cfg_attr(feature = "clap", clap(
        short, long,
        env = crate::env::LOG,
        value_name = "FILTER",
    ))]
    pub log: Option<String>,

    /// 2-bit color palette.
    ///
    /// Select from a list of preset 2-bit color palettes for the DMG model.
    /// Custom values can be defined in the configuration file.
    #[cfg_attr(
        feature = "clap",
        clap(short, long = "palette", value_name = "COLOR", value_enum)
    )]
    #[cfg_attr(feature = "serde", serde(rename = "palette"))]
    pub pal: Option<Palette>,

    /// Simulated clock speed.
    ///
    /// Select from a list of possible speeds to simulate the emulator's clock.
    /// Custom values can be defined in the configuration file.
    #[cfg_attr(
        feature = "clap",
        clap(short, long = "speed", value_name = "FREQ", value_enum)
    )]
    #[cfg_attr(feature = "serde", serde(rename = "speed"))]
    pub spd: Option<Speed>,
}

impl Conf for Application {
    fn rebase(&mut self, _: &Path) {}

    fn merge(&mut self, other: Self) {
        self.log = self.log.take().or(other.log);
        self.pal = self.pal.take().or(other.pal);
        self.spd = self.spd.take().or(other.spd);
    }
}
