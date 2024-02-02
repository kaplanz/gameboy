//! Generalized debugger prompt.

use std::fmt::Debug;

use thiserror::Error;

/// Behaviour for prompting a user for input.
pub trait Prompt: Debug + Send {
    /// Present the prompt message and receive a debugger command from the user.
    ///
    /// # Errors
    ///
    /// Returns an error if the prompt fails to produce input. If the user
    /// wishes to terminate the program, the special error [`Error::Quit`] will
    /// be returned.
    fn prompt(&mut self, msg: &str) -> Result<String, Error>;
}

/// A type specifying categories of [`Prompt`] errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Generic internal error.
    #[error(transparent)]
    Internal(#[from] Box<dyn std::error::Error>),
    /// Quit request.
    ///
    /// Special error used to signal to the debugger that the uesr has requested
    /// to quit the program.
    #[error("quit requested by user")]
    Quit,
}
