//! Screen color palettes.

use std::fmt::{Debug, Display};
use std::ops::Index;
use std::str::FromStr;

use serde::{Deserialize as De, Serialize as Ser};

/// Use [24-bit] color (stored as `0x00RRGGBB_u32`)
///
/// [24-bit]: https://en.wikipedia.org/wiki/List_of_monochrome_and_RGB_color_formats#24-bit_RGB
type Color = u32;

/// Color palette.
#[derive(Debug, Ser, De)]
pub struct Palette([Color; 4]);

impl Default for Palette {
    fn default() -> Self {
        Self([0xe9efec, 0xa0a08b, 0x555568, 0x211e20])
    }
}

impl Display for Palette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&json5::to_string(self).unwrap(), f)
    }
}

impl FromStr for Palette {
    type Err = json5::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        json5::from_str(s)
    }
}

impl Index<usize> for Palette {
    type Output = Color;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
