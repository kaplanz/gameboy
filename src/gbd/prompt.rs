use std::fmt::Debug;

use thiserror::Error;

/// Behaviour for prompting a user for input.
pub trait Prompt: Debug + Send {
    /// Prompt the user for input.
    ///
    /// # Errors
    ///
    /// Returns an error if the prompt fails to produce input. If the user
    /// wishes to terminate the program, the special error [`Error::Quit`] will
    /// be returned.
    fn prompt(&mut self, ask: &str) -> Result<String, Error>;
}

/// A type specifying categories of [`Prompt`] errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Internal(#[from] Box<dyn std::error::Error>),
    #[error("quit requested by user")]
    Quit,
}
