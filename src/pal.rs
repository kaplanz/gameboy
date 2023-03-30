//! Screen color palettes.

use std::fmt::{Debug, Display};
use std::ops::Index;
use std::str::FromStr;

use thiserror::Error;

/// Use [24-bit] color (stored as `0x00RRGGBB_u32`)
///
/// [24-bit]: https://en.wikipedia.org/wiki/List_of_monochrome_and_RGB_color_formats#24-bit_RGB
#[derive(Copy, Clone, Debug, Default)]
pub struct Color(u32);

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("#{:06x}", self.0), f)
    }
}

impl FromStr for Color {
    type Err = hexicolor::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Color(s.parse::<hexicolor::Color>()?.into()))
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        value.0
    }
}

/// Color palette.
#[derive(Clone, Debug)]
pub struct Palette([Color; 4]);

impl Default for Palette {
    fn default() -> Self {
        Self([
            Color(0x00e9_efec),
            Color(0x00a0_a08b),
            Color(0x0055_5568),
            Color(0x0021_1e20),
        ])
    }
}

impl Display for Palette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(
            &format!("{},{},{},{}", self.0[0], self.0[1], self.0[2], self.0[3]),
            f,
        )
    }
}

impl FromStr for Palette {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pal: Vec<Color> = match s
            .split(',')
            .map(str::parse::<Color>)
            .collect::<Result<_, _>>()
        {
            Ok(value) => value,
            Err(err) => return Err(Error::Parse(err)),
        };
        let pal: [Color; 4] = pal.try_into().map_err(|err: Vec<_>| match err.len() {
            len @ 0..=3 => Error::Missing(len),
            len @ 5.. => Error::Extra(len),
            _ => unreachable!(),
        })?;
        Ok(Palette(pal))
    }
}

impl Index<usize> for Palette {
    type Output = Color;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Parse(hexicolor::Error),
    #[error("missing palette colors: (found {0}, expected 4)")]
    Missing(usize),
    #[error("extra palette colors: (found {0}, expected 4)")]
    Extra(usize),
}
