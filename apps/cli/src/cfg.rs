//! Application configuration.

use std::io::ErrorKind::NotFound;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub use rugby_cfg::Config;
use thiserror::Error;
use toml::from_str as parse;

use crate::dir;

/// Returns the path to the application's configuration file.
#[must_use]
pub fn path() -> PathBuf {
    dir::config().join("config.toml")
}

/// Loads configuration data from a file.
///
/// # Errors
///
/// This function will return an error if the configuration could not be
/// loaded.
pub fn load(path: &Path) -> Result<Config> {
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

/// A convenient type alias for [`Result`](std::result::Result).
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error caused by [loading](load) the configuration.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to read config.
    #[error("failed to read config")]
    Read(#[from] io::Error),
    /// Failed to parse config.
    #[error("failed to parse config")]
    Parse(#[from] toml::de::Error),
}
