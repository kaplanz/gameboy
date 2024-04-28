use thiserror::Error;

use crate::api::video;
#[derive(Clone, Debug)]
pub struct Pixel {
    /// Color value.
    pub col: Color,
    /// Pixel metadata.
    pub meta: Meta,
}

impl Pixel {
    /// Constructs a new `Pixel`.
    pub fn new(col: Color, meta: Meta) -> Self {
        Self { col, meta }
    }

    /// Blends a pair of window/background and sprite pixels together.
    #[allow(clippy::if_same_then_else)]
    pub fn blend(winbg: Self, sprite: Self) -> Self {
        // Pixels are blended as follows:
        // - If the color number of the Sprite Pixel is 0, the Background Pixel
        //   is pushed to the LCD.
        if sprite.col == Color::C0 {
            winbg
        }
        // - If the BG-to-OBJ-Priority bit is 1 and the color number of the
        //   Background Pixel is anything other than 0, the Background Pixel is
        //   pushed to the LCD.
        else if sprite.meta.bgp && winbg.col != Color::C0 {
            winbg
        }
        // - If none of the above conditions apply, the Sprite Pixel is pushed
        //   to the LCD.
        else {
            sprite
        }
        // <https://hacktix.github.io/GBEDG/ppu/#pixel-mixing>
    }
}

/// Color values.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Color {
    /// Lightest
    #[default]
    C0 = 0b00,
    /// Light
    C1 = 0b01,
    /// Dark
    C2 = 0b10,
    /// Darkest
    C3 = 0b11,
}

impl Color {
    pub(crate) fn recolor(self, pal: u8) -> Self {
        Self::try_from((pal >> (2 * (self as u8))) & 0b11).unwrap()
    }
}

impl video::Pixel for Color {}

impl TryFrom<u8> for Color {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Self::C0),
            0b01 => Ok(Self::C1),
            0b10 => Ok(Self::C2),
            0b11 => Ok(Self::C3),
            _ => Err(Error::Color),
        }
    }
}

/// Pixel metadata.
#[derive(Clone, Copy, Debug)]
pub struct Meta {
    /// Color palette.
    pub pal: Palette,
    /// Background priority.
    pub bgp: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Palette {
    BgWin,
    Obp0,
    Obp1,
}

/// A convenient type alias for [`Result`](std::result::Result).
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A type specifying categories of [`Pixel`] error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown color")]
    Color,
}
