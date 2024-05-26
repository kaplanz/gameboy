use std::ops::{Deref, DerefMut};

use rugby_arch::Byte;

use super::pixel::Color;

#[derive(Clone, Debug)]
pub struct Tile([Row; 8]);

impl Tile {
    #[allow(unused)]
    pub fn xflip(&mut self) {
        self.0.iter_mut().for_each(Row::xflip);
    }

    #[allow(unused)]
    pub fn yflip(&mut self) {
        self.0.reverse();
    }
}

impl Deref for Tile {
    type Target = [Row; 8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<[Row; 8]> for Tile {
    fn from(rows: [Row; 8]) -> Self {
        Self(rows)
    }
}

impl From<[Byte; 16]> for Tile {
    fn from(bytes: [Byte; 16]) -> Self {
        let rows: [Row; 8] = bytes
            .chunks_exact(2)
            .map(|row| <[_; 2]>::try_from(row).unwrap())
            .map(Row::from)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        rows.into()
    }
}

#[derive(Clone, Debug)]
pub struct Row([Color; 8]);

impl Row {
    pub fn xflip(&mut self) {
        self.reverse();
    }
}

impl Deref for Row {
    type Target = [Color; 8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Row {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<[Byte; 2]> for Row {
    fn from(bytes: [Byte; 2]) -> Self {
        // Iterate through each bit of both bytes
        let pixels = (0..Byte::BITS)
            // Combine upper and lower bytes into colors for each bit
            .map(|bit| {
                // Extract color bits
                let mask = 0b1 << bit;
                let bit0 = bytes[0] & mask != 0;
                let bit1 = bytes[1] & mask != 0;
                // Combine into color value
                (Byte::from(bit1) << 1) | Byte::from(bit0)
            })
            // Reverse, since bit 7 represents the leftmost pixel, and bit 0 the
            // rightmost.
            .rev()
            // Convert into pixels
            .map(|col| col.try_into().unwrap()) // succeeds since values are 2-bit
            // Collect into an array of [Pixel; 8]
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(); // this will succeed on any sane platform where there are
                       // 8 bits in a byte. We won't support whatever archaic
                       // (or futuristic) platform does otherwise. Sorry :P

        Self(pixels)
    }
}
