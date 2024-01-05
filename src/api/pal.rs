//! Screen color palettes.

use std::fmt::Debug;
use std::ops::Index;

use chex::Color;
use serde::{Deserialize, Serialize};

/// 2-bit color palette.
///
/// Used by the DMG model; the 2-bit palette depth supports a total of 4 colors.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Palette([Color; 4]);

impl Index<usize> for Palette {
    type Output = Color;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[allow(clippy::unreadable_literal)]
mod decl {
    use super::{Color, Palette};

    /// Chrome palette.
    ///
    /// Based upon [2bit Demichrome][source] by [Space Sandwich][author].
    ///
    /// [author]: https://lospec.com/kerrielake
    /// [source]: https://lospec.com/palette-list/2bit-demichrome
    pub const CHROME: Palette = Palette([
        Color::new(0xe9efec),
        Color::new(0xa0a08b),
        Color::new(0x555568),
        Color::new(0x211e20),
    ]);

    /// Legacy palette.
    ///
    /// Based upon [Legacy][source] by [Patrick Adams][author].
    ///
    /// [author]: https://www.deviantart.com/thewolfbunny64
    /// [source]: https://www.deviantart.com/thewolfbunny64/art/Game-Boy-Palette-DMG-Ver-808181265
    pub const LEGACY: Palette = Palette([
        Color::new(0x7f860f),
        Color::new(0x577c44),
        Color::new(0x365d48),
        Color::new(0x2a453b),
    ]);

    /// Mystic palette.
    ///
    /// Based upon [Mist][source] by [Kerrie Lake][author].
    ///
    /// [author]: https://lospec.com/kerrielake
    /// [source]: https://lospec.com/palette-list/mist-gb
    pub const MYSTIC: Palette = Palette([
        Color::new(0xc4f0c2),
        Color::new(0x5ab9a8),
        Color::new(0x1e606e),
        Color::new(0x2d1b00),
    ]);

    /// Rustic palette.
    ///
    /// Based upon [Rustic][source] by [Kerrie Lake][author].
    ///
    /// [author]: https://lospec.com/kerrielake
    /// [source]: https://lospec.com/palette-list/rustic-gb
    pub const RUSTIC: Palette = Palette([
        Color::new(0xedb4a1),
        Color::new(0xa96868),
        Color::new(0x764462),
        Color::new(0x2c2137),
    ]);

    /// Winter palette.
    ///
    /// Based upon [BlueDream4][source] by [Snowy Owl][author].
    ///
    /// [author]: https://lospec.com/snowy-owl
    /// [source]: https://lospec.com/palette-list/bluedream4
    pub const WINTER: Palette = Palette([
        Color::new(0xecf2cb),
        Color::new(0x98d8b1),
        Color::new(0x4b849a),
        Color::new(0x1f285d),
    ]);
}

pub use self::decl::*;
