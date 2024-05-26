use rugby_arch::{Byte, Word};

use super::pixel::{Meta, Palette};

#[derive(Clone, Debug)]
pub struct Sprite {
    // Byte 0 - Y Position
    pub ypos: Byte,
    // Byte 1 - X Position
    pub xpos: Byte,
    // Byte 2 - Tile Index
    pub tidx: Byte,
    // Byte 3 - Attributes/Flags
    // Layout: 0b[Z, Y, X, P, 0000]
    // - P: palette
    // - X: x-flip
    // - Y: y-flip
    // - Z: priority
    pub pal: Palette,
    pub xflip: bool,
    pub yflip: bool,
    pub bgp: bool,
}

impl Sprite {
    pub fn meta(Self { pal, bgp, .. }: Self) -> Meta {
        Meta { pal, bgp }
    }

    pub fn yflip(addr: &mut Word) {
        *addr ^= 0b0000_1110;
    }
}

impl From<[Byte; 4]> for Sprite {
    #[rustfmt::skip]
    fn from(bytes: [Byte; 4]) -> Self {
        Self {
            ypos:  bytes[0],
            xpos:  bytes[1],
            tidx:  bytes[2],
            bgp:   bytes[3] & 0b1000_0000 != 0,
            yflip: bytes[3] & 0b0100_0000 != 0,
            xflip: bytes[3] & 0b0010_0000 != 0,
            pal: [
                Palette::Obp0,
                Palette::Obp1,
            ][usize::from(bytes[3] & 0b0001_0000 != 0)],
        }
    }
}
