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

use serde_with::{DeserializeFromStr, SerializeDisplay};
use thiserror::Error;

/// Use [24-bit] color (stored as `0x00RRGGBB_u32`)
///
/// [24-bit]: https://en.wikipedia.org/wiki/List_of_monochrome_and_RGB_color_formats#24-bit_RGB
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    DeserializeFromStr,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    SerializeDisplay,
)]
pub struct Color(u32);

impl Color {
    /// Constructs a new `Color` with the provided integer value.
    #[must_use]
    pub fn new(color: u32) -> Self {
        Self(color)
    }

    /// Constructs a new `Color` with the provided RGB values.
    #[must_use]
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(u32::from_be_bytes([0x00, r, g, b]))
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
                    3 => Ok({
                        let r = (hex & 0xf00) * 0x1100;
                        let g = (hex & 0x0f0) * 0x0110;
                        let b = (hex & 0x00f) * 0x0011;
                        Self(r | g | b)
                    }),
                    6 => Ok(Self(hex)),
                    _ => Err(Error::Unsupported),
                }
            }
            _ => Err(Error::MissingHash),
        }
    }
}

/// A type specifying categories of [`Color`] error.
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
    fn rgb_works() {
        assert_eq!(Color::rgb(0x11, 0x22, 0x33), Color(0x112233));
    }

    #[rustfmt::skip]
    #[test]
    fn parse_works() {
        assert_eq!(   "#123".parse::<Color>().unwrap(), Color(0x112233));
        assert_eq!("#123456".parse::<Color>().unwrap(), Color(0x123456));
        assert_eq!("#AbCdEf".parse::<Color>().unwrap(), Color(0xabcdef));
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
