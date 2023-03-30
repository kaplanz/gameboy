use super::pixel::{Meta, Palette};

#[derive(Clone, Debug)]
pub struct Sprite {
    // Byte 0 - Y Position
    pub ypos: u8,
    // Byte 1 - X Position
    pub xpos: u8,
    // Byte 2 - Tile Index
    pub idx: u8,
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

    pub fn yflip(addr: &mut u16) {
        *addr ^= 0b0000_1110;
    }
}

impl From<[u8; 4]> for Sprite {
    #[rustfmt::skip]
    fn from(bytes: [u8; 4]) -> Self {
        Self {
            ypos:     bytes[0],
            xpos:     bytes[1],
            idx:      bytes[2],
            bgp: bytes[3] & 0b1000_0000 != 0,
            yflip:    bytes[3] & 0b0100_0000 != 0,
            xflip:    bytes[3] & 0b0010_0000 != 0,
            pal: [
                Palette::Obp0,
                Palette::Obp1,
            ][usize::from(bytes[3] & 0b0001_0000 != 0)],
        }
    }
}
