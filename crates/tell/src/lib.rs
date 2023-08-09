pub extern crate owo_colors;

pub use log::Level;
use owo_colors::OwoColorize;

#[macro_export]
macro_rules! tell {
    ($lvl:expr, $($arg:tt)+) => {
        println!(
            "{level}{colon} {msg}",
            level = $crate::Tell::display($lvl),
            colon = $crate::owo_colors::OwoColorize::bold(&":"),
            msg = format!($($arg)+),
        )
    }
}

#[macro_export]
macro_rules! error {
    // error!("a {} event", "log")
    ($($arg:tt)+) => ($crate::tell!($crate::Level::Error, $($arg)+))
}

#[macro_export]
macro_rules! warn {
    // warn!("a {} event", "log")
    ($($arg:tt)+) => ($crate::tell!($crate::Level::Warn, $($arg)+))
}

#[macro_export]
macro_rules! info {
    // info!("a {} event", "log")
    ($($arg:tt)+) => ($crate::tell!($crate::Level::Info, $($arg)+))
}

#[macro_export]
macro_rules! debug {
    // debug!("a {} event", "log")
    ($($arg:tt)+) => ($crate::tell!($crate::Level::Debug, $($arg)+))
}

#[macro_export]
macro_rules! trace {
    // trace!("a {} event", "log")
    ($($arg:tt)+) => ($crate::tell!($crate::Level::Trace, $($arg)+))
}

pub trait Tell: private::Sealed {
    fn display(self) -> String;
}

impl Tell for Level {
    fn display(self) -> String {
        match self {
            Level::Error => self.to_string().to_lowercase().red().bold().to_string(),
            Level::Warn => self.to_string().to_lowercase().yellow().to_string(),
            Level::Info => self.to_string().to_lowercase().green().to_string(),
            Level::Debug => self.to_string().to_lowercase().blue().to_string(),
            Level::Trace => self.to_string().to_lowercase().magenta().to_string(),
        }
    }
}

mod private {
    use super::Level;

    pub trait Sealed {}

    impl Sealed for Level {}
}
