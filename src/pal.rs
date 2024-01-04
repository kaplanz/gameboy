//! Screen color palettes.

use std::fmt::{Debug, Display};
use std::ops::Index;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

mod decl {
    #![allow(clippy::unreadable_literal)]

    use super::{Color, Palette};

    /// Chrome palette.
    ///
    /// Based upon [2bit Demichrome][source] by [Space Sandwich][author].
    ///
    /// [author]: https://lospec.com/kerrielake
    /// [source]: https://lospec.com/palette-list/2bit-demichrome
    pub const CHROME: Palette = Palette([
        Color(0xe9efec),
        Color(0xa0a08b),
        Color(0x555568),
        Color(0x211e20),
    ]);

    /// Legacy palette.
    ///
    /// Based upon [Legacy][source] by [Patrick Adams][author].
    ///
    /// [author]: https://www.deviantart.com/thewolfbunny64
    /// [source]: https://www.deviantart.com/thewolfbunny64/art/Game-Boy-Palette-DMG-Ver-808181265
    pub const LEGACY: Palette = Palette([
        Color(0x7f860f),
        Color(0x577c44),
        Color(0x365d48),
        Color(0x2a453b),
    ]);

    /// Mystic palette.
    ///
    /// Based upon [Mist][source] by [Kerrie Lake][author].
    ///
    /// [author]: https://lospec.com/kerrielake
    /// [source]: https://lospec.com/palette-list/mist-gb
    pub const MYSTIC: Palette = Palette([
        Color(0xc4f0c2),
        Color(0x5ab9a8),
        Color(0x1e606e),
        Color(0x2d1b00),
    ]);

    /// Rustic palette.
    ///
    /// Based upon [Rustic][source] by [Kerrie Lake][author].
    ///
    /// [author]: https://lospec.com/kerrielake
    /// [source]: https://lospec.com/palette-list/rustic-gb
    pub const RUSTIC: Palette = Palette([
        Color(0xedb4a1),
        Color(0xa96868),
        Color(0x764462),
        Color(0x2c2137),
    ]);

    /// Winter palette.
    ///
    /// Based upon [BlueDream4][source] by [Snowy Owl][author].
    ///
    /// [author]: https://lospec.com/snowy-owl
    /// [source]: https://lospec.com/palette-list/bluedream4
    pub const WINTER: Palette = Palette([
        Color(0xecf2cb),
        Color(0x98d8b1),
        Color(0x4b849a),
        Color(0x1f285d),
    ]);
}

pub use self::decl::*;

/// 2-bit color palette.
///
/// Used for the DMG model console; the 2-bit color depth supports a total of
/// four colors.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Palette([Color; 4]);

impl Index<usize> for Palette {
    type Output = Color;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

/// Use [24-bit] color (stored as `0x00RRGGBB_u32`)
///
/// [24-bit]: https://en.wikipedia.org/wiki/List_of_monochrome_and_RGB_color_formats#24-bit_RGB
#[derive(Copy, Clone, Debug, Default, DeserializeFromStr, SerializeDisplay)]
pub struct Color(u32);

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&format!("#{:06x}", self.0), f)
    }
}

impl FromStr for Color {
    type Err = chex::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse::<chex::Color>()?.into()))
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        value.0
    }
}
