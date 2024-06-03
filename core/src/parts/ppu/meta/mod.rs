//! Graphics metadata.

mod color;
mod obj;
mod pixel;
mod tile;

pub use color::Color;
pub use obj::{Attributes, Sprite};
pub use pixel::{Meta, Pixel};
pub use tile::{Row, Tile};

/// Graphics layer.
///
/// The Game Boy has three different graphical layers that are used to display
/// pixels within the 160x144 viewport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Layer {
    /// The Background is a 256x256 grid of pixels (32x32 grid of
    /// [tiles](Tile)).
    Background,
    /// The Window is a 256x256 grid of pixels (32x32 [tiles](Tile)) which is
    /// used as a fixed overlay over the [background](Layer::Background).
    Window,
    /// Sprites are 8x8 (or occasionally 8x16) pixel tile(s) which can be
    /// rendered freely anywhere within the viewport.
    Sprite,
}

/// Monochrome palettes.
///
/// This register reassigns color values to pixels according to how palette
/// registers are configured using the following layout:
///
/// | [7:6] | [5:4] | [3:2] | [1:0] |
/// |-------|-------|-------|-------|
/// | `C3`  | `C2`  | `C1`  | `C0`  |
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Palette {
    /// Background/Window palette.
    BgWin,
    /// Object palette 0.
    Obp0,
    /// Object palette 1.
    Obp1,
}

impl Palette {
    /// Constructs a new sprite `Palette`.
    #[must_use]
    pub fn obp(flag: bool) -> Self {
        [Self::Obp0, Self::Obp1][usize::from(flag)]
    }
}
