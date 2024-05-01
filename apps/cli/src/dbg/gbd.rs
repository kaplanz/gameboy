//! Game Boy Debugger (GBD).

use std::error::Error as StdError;
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use gbd::prompt::{Error, Prompt};
use log::{debug, error, trace};
use rustyline::error::ReadlineError::{Eof, Interrupted as Int};
use rustyline::history::History;
use rustyline::DefaultEditor as Editor;

use crate::dir;

/// Returns the path to the application's history file.
#[must_use]
pub fn history() -> PathBuf {
    dir::state().join("history.txt")
}

/// Interface over the user's console.
#[derive(Debug)]
pub struct Console {
    edit: Editor,
}

impl Console {
    /// Constructs a new `Console`.
    pub fn new() -> anyhow::Result<Self> {
        Self {
            edit: Editor::new()?,
        }
        .init()
    }

    /// Initializes the console.
    fn init(mut self) -> anyhow::Result<Self> {
        // Create state directory
        let state = dir::state();
        if !state.exists() {
            fs::create_dir_all(&state)
                .inspect(|()| trace!("created state directory: `{}`", state.display()))
                .with_context(|| format!("failed to create directory: `{}`", state.display()))?;
        }
        // Set maximum history entries
        self.edit.history_mut().set_max_len(10_000)?;
        // Load previous history from file
        self.load()?;
        // Return initialized console
        Ok(self)
    }

    /// Loads history from a file into the prompt.
    fn load(&mut self) -> anyhow::Result<()> {
        // Get histfile path
        let path = self::history();
        if !path.exists() {
            // Don't read any history if the file does not (yet) exist. Instead,
            // return without doing anything.
            return Ok(());
        }
        // Read history from file
        debug!("loading history: `{}`", path.display());
        self.edit
            .load_history(&path)
            .inspect(|()| {
                debug!(
                    "loaded history: {} items",
                    self.edit.history().iter().count()
                );
                trace!(
                    "history: {:#?}",
                    self.edit.history().iter().collect::<Vec<_>>()
                );
            })
            .context("failed to load history")
    }

    /// Saves history from the prompt into a file.
    fn save(&mut self) -> anyhow::Result<()> {
        // Get histfile path
        let path = self::history();
        // Write history to file
        debug!("saving history: `{}`", path.display());
        self.edit
            .save_history(&path)
            .inspect(|()| {
                debug!(
                    "saved history: {} items",
                    self.edit.history().iter().count()
                );
                trace!(
                    "history: {:#?}",
                    self.edit.history().iter().collect::<Vec<_>>()
                );
            })
            .context("failed to save history")
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        if let Err(err) = self.save() {
            error!("{err:#}");
        }
    }
}

impl Prompt for Console {
    fn prompt(&mut self, msg: &str) -> Result<String, Error> {
        // Prompt the user for input
        let line = loop {
            match self.edit.readline(msg) {
                Ok(line) => break line,
                Err(err) => match err {
                    Int => continue,
                    Eof => return Err(Error::Quit),
                    _ => return Err(Error::Internal(Box::new(err))),
                },
            }
        };
        // Add obtained input to history
        self.edit
            .history_mut()
            .add(&line)
            .map_err(|err| Box::new(err) as Box<dyn StdError>)?;
        // Return user input
        Ok(line)
    }
}
