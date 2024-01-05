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

    /// Autumn Chill.
    ///
    /// Nostalgic autumn sunsets.
    ///
    /// Based upon [Autumn Chill][source] by [Doph][author].
    ///
    /// [author]: https://lospec.com/dophsart
    /// [source]: https://lospec.com/palette-list/autumn-chill
    pub const AUTUMN_CHILL: Palette = Palette([
        Color::new(0xdad3af),
        Color::new(0xd58863),
        Color::new(0xc23a73),
        Color::new(0x2c1e74),
    ]);

    /// Blk Aqu.
    ///
    /// Aquatic blues.
    ///
    /// Based upon [Blk Aqu][source] by [BurakoIRL][author].
    ///
    /// [author]: https://lospec.com/blkirl
    /// [source]: https://lospec.com/palette-list/blk-aqu4
    pub const BLK_AQU: Palette = Palette([
        Color::new(0x9ff4e5),
        Color::new(0x00b9be),
        Color::new(0x005f8c),
        Color::new(0x002b59),
    ]);

    /// Blue Dream.
    ///
    /// Winter snowstorm blues.
    ///
    /// Based upon [Blue Dream][source] by [Snowy Owl][author].
    ///
    /// [author]: https://lospec.com/snowy-owl
    /// [source]: https://lospec.com/palette-list/bluedream4
    pub const BLUE_DREAM: Palette = Palette([
        Color::new(0xecf2cb),
        Color::new(0x98d8b1),
        Color::new(0x4b849a),
        Color::new(0x1f285d),
    ]);

    /// Coldfire.
    ///
    /// Combining cool and warm tones.
    ///
    /// Based upon [Coldfire][source] by [Kerrie Lake][author].
    ///
    /// [author]: https://lospec.com/kerrielake
    /// [source]: https://lospec.com/palette-list/coldfire-gb
    pub const COLDFIRE: Palette = Palette([
        Color::new(0xf6c6a8),
        Color::new(0xd17c7c),
        Color::new(0x5b768d),
        Color::new(0x46425e),
    ]);

    /// Coral.
    ///
    /// Soft and pastel coral hues.
    ///
    /// Based upon [Coral][source] by [Yousurname][author].
    ///
    /// [author]: https://lospec.com/yousurname
    /// [source]: https://lospec.com/palette-list/coral-4
    pub const CORAL: Palette = Palette([
        Color::new(0xffd0a4),
        Color::new(0xf4949c),
        Color::new(0x7c9aac),
        Color::new(0x68518a),
    ]);

    /// Demichrome.
    ///
    /// Cold metallic darks with warm dated plastic lights.
    ///
    /// Based upon [2bit Demichrome][source] by [Space Sandwich][author].
    ///
    /// [author]: https://lospec.com/spacesandwich
    /// [source]: https://lospec.com/palette-list/2bit-demichrome
    pub const DEMICHROME: Palette = Palette([
        Color::new(0xe9efec),
        Color::new(0xa0a08b),
        Color::new(0x555568),
        Color::new(0x211e20),
    ]);

    /// Earth.
    ///
    /// Greens and warm browns with an earthy feel.
    ///
    /// Based upon [Earth][source] by [Kerrie Lake][author].
    ///
    /// [author]: https://lospec.com/kerrielake
    /// [source]: https://lospec.com/palette-list/earth-gb
    pub const EARTH: Palette = Palette([
        Color::new(0xf5f29e),
        Color::new(0xacb965),
        Color::new(0xb87652),
        Color::new(0x774346),
    ]);

    /// Ice Cream.
    ///
    /// Creamsicle inspired orange.
    ///
    /// Based upon [Ice Cream][source] by [Kerrie Lake][author].
    ///
    /// [author]: https://lospec.com/kerrielake
    /// [source]: https://lospec.com/palette-list/ice-cream-gb
    pub const ICE_CREAM: Palette = Palette([
        Color::new(0xfff6d3),
        Color::new(0xf9a875),
        Color::new(0xeb6b6f),
        Color::new(0x7c3f58),
    ]);

    /// Legacy.
    ///
    /// Old-school dot-matrix display.
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

    /// Mist.
    ///
    /// Misty forest greens.
    ///
    /// Based upon [Mist][source] by [Kerrie Lake][author].
    ///
    /// [author]: https://lospec.com/kerrielake
    /// [source]: https://lospec.com/palette-list/mist-gb
    pub const MIST: Palette = Palette([
        Color::new(0xc4f0c2),
        Color::new(0x5ab9a8),
        Color::new(0x1e606e),
        Color::new(0x2d1b00),
    ]);

    /// Mono.
    ///
    /// Simple blacks and whites.
    pub const MONO: Palette = Palette([
        Color::new(0xffffff),
        Color::new(0xaaaaaa),
        Color::new(0x555555),
        Color::new(0x000000),
    ]);

    /// Morris.
    ///
    /// William Morris's rural palette.
    ///
    /// Based upon [Morris][source] by [Rabbit King][author].
    ///
    /// [author]: https://lospec.com/rabbitking
    /// [source]: https://lospec.com/palette-list/gb-morris
    pub const MORRIS: Palette = Palette([
        Color::new(0xe5d8ac),
        Color::new(0x7db3ab),
        Color::new(0x7c714a),
        Color::new(0x264b38),
    ]);

    /// Purple Dawn.
    ///
    /// Waterfront at dawn.
    ///
    /// Based upon [Purple Dawn][source] by [WildLeoKnight][author].
    ///
    /// [author]: https://lospec.com/wildleoknight
    /// [source]: https://lospec.com/palette-list/purpledawn
    pub const PURPLE_DAWN: Palette = Palette([
        Color::new(0xeefded),
        Color::new(0x9a7bbc),
        Color::new(0x2d757e),
        Color::new(0x001b2e),
    ]);

    /// Rustic.
    ///
    /// Rusty red and brown hues.
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

    /// Velvet Cherry.
    ///
    /// Deep and passionate purples.
    ///
    /// Based upon [Velvet Cherry][source] by [Klafooty][author].
    ///
    /// [author]: https://lospec.com/mallory
    /// [source]: https://lospec.com/palette-list/velvet-cherry-gb
    pub const VELVET_CHERRY: Palette = Palette([
        Color::new(0x9775a6),
        Color::new(0x683a68),
        Color::new(0x412752),
        Color::new(0x2d162c),
    ]);
}

pub use self::decl::*;
