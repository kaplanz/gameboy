//! Hex color parser.
//!
//! This library implements a string color parser for hexadecimal color values.
//!
//! # Examples
//!
//! ```
//! use chex::Color;
//!
//! # fn main() -> Result<(), chex::Error> {
//! // Parse color from a string
//! let col: Color = "#a0a08b".parse()?;
//!
//! // Convert color as its integer value
//! assert_eq!(u32::from(col) / 3, 0x358ad9);
//!
//! // Format back as a string
//! assert_eq!(format!("{col}"), "#a0a08b");
//! #
//! # Ok(())
//! # }
//! ```

#![warn(clippy::pedantic)]

use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

use thiserror::Error;

/// 32-bit color.
#[derive(Copy, Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Color(u32);

impl Color {
    /// Constructs a new [`Color`] with the provided integer value.
    #[must_use]
    pub fn new(color: u32) -> Self {
        Self(color)
    }

    /// Constructs a new [`Color`] with the provided RGB values.
    #[must_use]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b))
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("#{:06x}", self.0), f)
    }
}

impl From<u32> for Color {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        color.0
    }
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next().ok_or(Error::Empty)? {
            '#' => {
                let input = s.get(1..).ok_or(Error::Unsupported)?;
                let hex = u32::from_str_radix(input, 16).map_err(Error::ParseInt)?;
                match input.len() {
                    3 => Ok(Self(hex << 12 | hex)),
                    6 => Ok(Self(hex)),
                    _ => Err(Error::Unsupported),
                }
            }
            _ => Err(Error::MissingHash),
        }
    }
}

/// A type specifying general categories of [`Color`] error.
#[derive(Clone, Debug, Error)]
pub enum Error {
    /// Parse string was empty.
    #[error("could not parse empty string")]
    Empty,
    /// Parse string does not start with `#`
    #[error("must start with \"#\"")]
    MissingHash,
    /// Error parsing hexadecimal from input.
    #[error("could not parse hex")]
    ParseInt(#[from] ParseIntError),
    /// Use of an unsupported format.
    #[error("unsupported color format")]
    Unsupported,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_works() {
        assert_eq!("#123".parse::<Color>().unwrap(), Color(0x0012_3123));
        assert_eq!("#123456".parse::<Color>().unwrap(), Color(0x0012_3456));
        assert_eq!("#AbCdEf".parse::<Color>().unwrap(), Color(0x00ab_cdef));
    }

    #[test]
    fn parse_errors() {
        assert!(matches!("".parse::<Color>().unwrap_err(), Error::Empty));
        assert!(matches!(
            "shalom".parse::<Color>().unwrap_err(),
            Error::MissingHash
        ));
        assert!(matches!(
            "#".parse::<Color>().unwrap_err(),
            Error::ParseInt(_)
        ));
        assert!(matches!(
            "#shalom".parse::<Color>().unwrap_err(),
            Error::ParseInt(_)
        ));
        assert!(matches!(
            "#12".parse::<Color>().unwrap_err(),
            Error::Unsupported
        ));
        assert!(matches!(
            "#1234".parse::<Color>().unwrap_err(),
            Error::Unsupported
        ));
        assert!(matches!(
            "#1234567".parse::<Color>().unwrap_err(),
            Error::Unsupported
        ));
    }
}
