use super::{Attributes, Color, Palette};

/// Pre-rendered pixel.
#[derive(Clone, Debug)]
pub struct Pixel {
    /// Color value.
    pub col: Color,
    /// Pixel metadata.
    pub meta: Meta,
}

impl Pixel {
    /// Constructs a new `Pixel`.
    #[must_use]
    pub fn new(col: Color, meta: Meta) -> Self {
        Self { col, meta }
    }

    /// Blends a pair of window/background and sprite pixels together.
    #[allow(clippy::if_same_then_else)]
    #[must_use]
    pub fn blend(winbg: Self, sprite: Self) -> Self {
        // Pixels are blended as follows:
        //
        // 1. If the color number of the sprite pixel is 0, the background pixel
        //    is pushed to the LCD.
        if sprite.col == Color::C0 {
            winbg
        }
        // 2. If the BG-to-OBJ priority bit is 1 and the color number of the
        //    background pixel is anything other than 0, the background pixel is
        //    pushed to the LCD.
        else if sprite.meta.bgp && winbg.col != Color::C0 {
            winbg
        }
        // 3. If none of the above conditions apply, the Sprite Pixel is pushed
        //    to the LCD.
        else {
            sprite
        }
        // <https://hacktix.github.io/GBEDG/ppu/#pixel-mixing>
    }
}

/// Pixel metadata.
#[derive(Clone, Debug)]
pub struct Meta {
    /// Monochrome palette.
    pub pal: Palette,
    /// Background priority.
    pub bgp: bool,
}

impl Meta {
    /// Constructs background metadata.
    #[must_use]
    pub fn bgwin() -> Self {
        Self {
            pal: Palette::BgWin,
            bgp: false,
        }
    }

    /// Constructs object metadata.
    #[must_use]
    pub fn sprite(attr: &Attributes) -> Self {
        Self {
            pal: Palette::obp(attr.objp),
            bgp: attr.prty,
        }
    }
}
