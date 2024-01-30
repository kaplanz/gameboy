use std::error::Error as StdError;

use gameboy::gbd::prompt::{Error, Prompt};
use rustyline::error::ReadlineError;
use rustyline::history::History;
use rustyline::DefaultEditor as Editor;

#[derive(Debug)]
pub struct Readline(Editor);

impl Readline {
    /// Constructs a new `Terminal`.
    pub fn new() -> Result<Self, ReadlineError> {
        Editor::new().map(Self)
    }
}

impl Prompt for Readline {
    fn prompt(&mut self, ask: &str) -> Result<String, Error> {
        // Prompt the user for input
        let line = loop {
            match self.0.readline(ask) {
                Ok(line) => break line,
                Err(err) => match err {
                    ReadlineError::Interrupted => continue,
                    ReadlineError::Eof => return Err(Error::Quit),
                    _ => return Err(Error::Internal(Box::new(err))),
                },
            }
        };
        // Add obtained input to history
        self.0
            .history_mut()
            .add(&line)
            .map_err(|err| Box::new(err) as Box<dyn StdError>)?;
        // Return user input
        Ok(line)
    }
}
