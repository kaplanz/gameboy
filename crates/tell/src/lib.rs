//! User-level information printing.
//!
//! # Examples
//!
//! ```
//! use tell::{debug, error, info, trace, warn};
//!
//! // Inform the user of various message types
//! trace!("Here's a really specific thing that happened...");
//! debug!("This should help you find the problem.");
//! info!("For your information...");
//! warn!("Are you sure you want to do that?");
//! error!("It broke. Don't say you weren't warned!")
//! ```

#![warn(clippy::pedantic)]

pub extern crate owo_colors;

pub use log::Level;
use owo_colors::OwoColorize;

/// The standard informing macro.
///
/// This macro will generically tell with the specified [`Level`] and [`format!`]
/// based argument list.
///
/// # Examples
///
/// ```
/// use tell::{tell, Level};
///
/// # fn main() {
/// let data = (42, "Forty-two");
/// let private_data = "private";
///
/// tell!(Level::Error, "Received errors: {}, {}", data.0, data.1);
/// # }
/// ```
#[macro_export]
macro_rules! tell {
    ($lvl:expr, $($arg:tt)+) => {
        println!(
            "{level}{colon} {msg}",
            level = $crate::Tell::to_string(&$lvl),
            colon = $crate::owo_colors::OwoColorize::bold(&":"),
            msg = format!($($arg)+),
        )
    }
}

/// Informs with a message at the error level.
///
/// # Examples
///
/// ```
/// use tell::error;
///
/// # fn main() {
/// let (err_info, port) = ("No connection", 22);
///
/// error!("Error: {} on port {}", err_info, port);
/// # }
/// ```
#[macro_export]
macro_rules! error {
    // error!("a {} event", "log")
    ($($arg:tt)+) => ($crate::tell!($crate::Level::Error, $($arg)+))
}

/// Informs with a message at the warn level.
///
/// # Examples
///
/// ```
/// use tell::warn;
///
/// # fn main() {
/// let warn_description = "Invalid Input";
///
/// warn!("Warning! {}!", warn_description);
/// # }
/// ```
#[macro_export]
macro_rules! warn {
    // warn!("a {} event", "log")
    ($($arg:tt)+) => ($crate::tell!($crate::Level::Warn, $($arg)+))
}

/// Informs with a message at the info level.
///
/// # Examples
///
/// ```
/// use tell::info;
///
/// # fn main() {
/// # struct Connection { port: u32, speed: f32 }
/// let conn_info = Connection { port: 40, speed: 3.20 };
///
/// info!("Connected to port {} at {} Mb/s", conn_info.port, conn_info.speed);
/// # }
/// ```
#[macro_export]
macro_rules! info {
    // info!("a {} event", "log")
    ($($arg:tt)+) => ($crate::tell!($crate::Level::Info, $($arg)+))
}

/// Informs with a message at the debug level.
///
/// # Examples
///
/// ```
/// use tell::debug;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// debug!("New position: x: {}, y: {}", pos.x, pos.y);
/// # }
/// ```
#[macro_export]
macro_rules! debug {
    // debug!("a {} event", "log")
    ($($arg:tt)+) => ($crate::tell!($crate::Level::Debug, $($arg)+))
}

/// Informs with a message at the trace level.
///
/// # Examples
///
/// ```
/// use tell::trace;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// trace!("Position is: x: {}, y: {}", pos.x, pos.y);
/// # }
/// ```
#[macro_export]
macro_rules! trace {
    // trace!("a {} event", "log")
    ($($arg:tt)+) => ($crate::tell!($crate::Level::Trace, $($arg)+))
}

/// Sealed trait providing a colored representation for [`Level`].
pub trait Tell: private::Sealed {
    /// Converts the given value to a [`String`].
    fn to_string(&self) -> String;
}

impl Tell for Level {
    fn to_string(&self) -> String {
        match self {
            Level::Error => ToString::to_string(&self)
                .to_lowercase()
                .red()
                .bold()
                .to_string(),
            Level::Warn => ToString::to_string(&self)
                .to_string()
                .to_lowercase()
                .yellow()
                .to_string(),
            Level::Info => ToString::to_string(&self)
                .to_string()
                .to_lowercase()
                .green()
                .to_string(),
            Level::Debug => ToString::to_string(&self)
                .to_string()
                .to_lowercase()
                .blue()
                .to_string(),
            Level::Trace => ToString::to_string(&self)
                .to_string()
                .to_lowercase()
                .magenta()
                .to_string(),
        }
    }
}

mod private {
    use super::Level;

    pub trait Sealed {}

    impl Sealed for Level {}
}
