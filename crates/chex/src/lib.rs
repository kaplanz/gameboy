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
#![allow(clippy::unreadable_literal)]

use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

use serde_with::{DeserializeFromStr, SerializeDisplay};
use thiserror::Error;

mod names;

/// [24-bit] color value.
///
/// Stored internally in a `u32` as `0x00RRGGBB`.
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
    pub const fn new(color: u32) -> Self {
        Self(color)
    }

    /// Constructs a new `Color` with the provided RGB values.
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self(u32::from_be_bytes([0x00, r, g, b]))
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("#{:06x}", self.0).fmt(f)
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
        if s.is_empty() {
            Err(Error::Empty)
        } else if s.starts_with('#') {
            let hex = s.get(1..).ok_or(Error::Unsupported)?;
            let val = u32::from_str_radix(hex, 16)?;
            match hex.len() {
                3 => Ok({
                    let r = (val & 0xf00) * 0x1100;
                    let g = (val & 0x0f0) * 0x0110;
                    let b = (val & 0x00f) * 0x0011;
                    Self(r | g | b)
                }),
                6 => Ok(Self(val)),
                _ => Err(Error::Unsupported),
            }
        } else if let Some(color) = names::COLORS.get(s) {
            Ok(*color)
        } else {
            Err(Error::Unknown(s.to_string()))
        }
    }
}

/// A convenient type alias for [`Result`](std::result::Result).
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A type specifying categories of [`Color`] error.
#[derive(Clone, Debug, Error)]
pub enum Error {
    /// Parse string was empty.
    #[error("empty string")]
    Empty,
    /// Error parsing value from input.
    #[error("failed to parse integer")]
    ParseInt(#[from] ParseIntError),
    /// Unknown color name.
    #[error("unknown color name: {0}")]
    Unknown(String),
    /// Unsupported hex format.
    #[error("unsupported hex format")]
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
        assert_eq!("crimson".parse::<Color>().unwrap(), Color(0xdc143c));
    }

    #[rustfmt::skip]
    #[test]
    fn parse_errors() {
        assert!(matches!(        "".parse::<Color>(), Err(Error::Empty)));
        assert!(matches!(  "shalom".parse::<Color>(), Err(Error::Unknown(_))));
        assert!(matches!(  "#error".parse::<Color>(), Err(Error::ParseInt(_))));
        assert!(matches!(       "#".parse::<Color>(), Err(Error::ParseInt(_))));
        assert!(matches!(     "#12".parse::<Color>(), Err(Error::Unsupported)));
        assert!(matches!(   "#1234".parse::<Color>(), Err(Error::Unsupported)));
        assert!(matches!("#1234567".parse::<Color>(), Err(Error::Unsupported)));
    }
}
