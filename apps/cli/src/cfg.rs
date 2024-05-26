//! Application configuration.

use std::io::ErrorKind::NotFound;
use std::path::{Path, PathBuf};
use std::{fs, io};

use clap::Args;
use serde::Deserialize;
use thiserror::Error;
use toml::from_str as parse;

use crate::dir;
use crate::opt::{Cartridge, Hardware, Interface};

/// Returns the path to the application's configuration file.
#[must_use]
pub fn path() -> PathBuf {
    dir::config().join("config.toml")
}

/// Configuration options.
#[derive(Args, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Cartridge options.
    #[clap(flatten)]
    #[clap(next_help_heading = "Cartridge")]
    #[serde(rename = "cartridge")]
    pub sw: Cartridge,

    /// Hardware options.
    #[clap(flatten)]
    #[clap(next_help_heading = "Hardware")]
    #[serde(rename = "hardware")]
    pub hw: Hardware,

    /// Interface options.
    #[clap(flatten)]
    #[clap(next_help_heading = "Interface")]
    #[serde(rename = "interface")]
    pub ui: Interface,
}

impl Config {
    /// Loads configuration data from a file.
    ///
    /// # Errors
    ///
    /// This function will return an error if the configuration could not be
    /// loaded.
    pub fn load(path: &Path) -> Result<Self> {
        match fs::read_to_string(path) {
            // If the configuration file does not exist, return an empty string,
            // resulting in all fields being populated with defaults.
            Err(err) if err.kind() == NotFound => Ok(String::default()),
            // For other errors, return them directly.
            Err(err) => Err(err.into()),
            // On success, return the body of the file can be parsed.
            Ok(body) => Ok(body),
        }
        .and_then(|body| {
            // If a configuration file was read, parse it.
            parse(&body)
                // Parsing errors should be mapped into a separate variant.
                .map_err(Into::into)
        })
    }

    /// Rebase relative paths to the provided root.
    ///
    /// Any relative paths will have be rebased such that they are not relative
    /// to the provided root.
    pub fn rebase(&mut self, root: &Path) {
        self.hw.rebase(root);
        self.sw.rebase(root);
        self.ui.rebase(root);
    }

    /// Combines two configuration instances.
    ///
    /// This is useful when some configurations may also be supplied on the
    /// command-line. When merging, it is best practice to prioritize options
    /// from the cli to those saved on-disk. To do so, prefer keeping data
    /// fields from `self` when conflicting with `other`.
    pub fn merge(&mut self, other: Self) {
        self.hw.merge(other.hw);
        self.sw.merge(other.sw);
        self.ui.merge(other.ui);
    }
}

/// A convenient type alias for [`Result`](std::result::Result).
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error caused by [loading][`Config::load`] the configuration.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to read config.
    #[error("failed to read config")]
    Read(#[from] io::Error),
    /// Failed to parse config.
    #[error("failed to parse config")]
    Parse(#[from] toml::de::Error),
}
