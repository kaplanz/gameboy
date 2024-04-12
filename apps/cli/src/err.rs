//! Error types.

use std::process::{ExitCode, Termination};

use thiserror::Error;

use crate::cfg;

/// A convenient type alias for application errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A top-level error caused within the application.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Configuration errors.
    #[error(transparent)]
    Config(#[from] cfg::Error),
    /// Catchall error variant.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<Error> for ExitCode {
    fn from(err: Error) -> Self {
        match err {
            Error::Config(_) => sysexits::ExitCode::Config.into(),
            _ => ExitCode::FAILURE,
        }
    }
}

/// Application exit conditions.
///
/// In the `Termination` implementation for `Exit`, we print any errors that
/// occur for the user.
#[derive(Debug)]
pub enum Exit {
    /// Exit success.
    Success,
    /// Exit failure.
    ///
    /// Advises the user about the [error][`enum@Error`], returning a non-zero
    /// [exit code][`ExitCode`].
    Failure(Error),
}

impl<E: Into<Error>> From<E> for Exit {
    fn from(err: E) -> Self {
        Self::Failure(err.into())
    }
}

impl Termination for Exit {
    fn report(self) -> ExitCode {
        match self {
            Exit::Success => ExitCode::SUCCESS,
            Exit::Failure(err) => {
                advise::error!("{err:#}");
                err.into()
            }
        }
    }
}
