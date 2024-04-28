//! Window graphics.

use thiserror::Error;

#[cfg(feature = "win")]
use self::dbg::Debug;

mod imp;

#[cfg(feature = "win")]
pub mod dbg;

pub use self::imp::{Aspect, Window};

/// Graphics window groups.
#[derive(Debug)]
pub struct Graphics {
    /// Main window.
    pub lcd: Window,
    /// Debug windows.
    #[cfg(feature = "win")]
    pub dbg: Debug,
}

impl Graphics {
    /// Constructs a new `Graphics`.
    pub fn new(title: &str, size: Aspect) -> Result<Self> {
        Ok(Self {
            lcd: Window::new(title, size)?,
            #[cfg(feature = "win")]
            dbg: Debug::default(),
        })
    }

    /// Checks if the frontend is alive.
    pub fn alive(&self) -> bool {
        self.lcd.is_open()
    }
}

/// A convenient type alias for [`Result`](std::result::Result).
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A type specifying categories of [`Graphics`] errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to render a window.
    #[error(transparent)]
    Window(#[from] imp::Error),
}
