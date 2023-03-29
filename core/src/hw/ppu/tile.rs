use std::ops::{Deref, DerefMut};

use super::pixel::{Palette, Pixel};

#[derive(Debug)]
pub struct Tile {
    pub rows: [Row; 8],
    pub pal: Palette,
}

impl Tile {
    #[allow(unused)]
    pub fn xflip(&mut self) {
        self.rows.iter_mut().for_each(|row| row.reverse());
    }

    #[allow(unused)]
    pub fn yflip(&mut self) {
        self.rows.reverse();
    }
}

#[derive(Debug)]
pub struct Row([Pixel; 8]);

impl Deref for Row {
    type Target = [Pixel; 8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Row {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<[u8; 2]> for Row {
    fn from(bytes: [u8; 2]) -> Self {
        // Iterate through each bit of both bytes
        let pixels = (0..u8::BITS)
            // Combine upper and lower bytes into colors for each bit
            .map(|bit| {
                // Extract color bits
                let mask = 0b1 << bit;
                let bit0 = bytes[0] & mask != 0;
                let bit1 = bytes[1] & mask != 0;
                // Combine into color value
                (u8::from(bit1) << 1) | u8::from(bit0)
            })
            // Reverse, since bit 7 represents the leftmost pixel, and bit 0 the
            // rightmost.
            .rev()
            // Convert into pixels
            .map(|col| Pixel {
                col: col.try_into().unwrap(), // succeeds since values are 2-bit
                // FIXME: Properly handle `pal`, `bgp`
                pal: Palette::BgWin,
                bgp: false,
            })
            // Collect into an array of [Pixel; 8]
            .collect::<Vec<Pixel>>()
            .try_into()
            .unwrap(); // this will succeed on any sane platform where there are
                       // 8 bits in a byte. We won't support whatever archaic
                       // (or futuristic) platform does otherwise. Sorry :P

        Self(pixels)
    }
}
