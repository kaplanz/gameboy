use std::ops::{Index, IndexMut};

use rugby_arch::Byte;

use super::Color;

/// 8x8 pixel tile.
#[derive(Clone, Debug)]
pub struct Tile([Row; 8]);

impl Tile {
    /// Horizontally flip a tile.
    pub fn xflip(&mut self) {
        self.0.iter_mut().for_each(Row::xflip);
    }

    /// Vertically flip a tile.
    pub fn yflip(&mut self) {
        self.0.reverse();
    }
}

impl From<[Row; 8]> for Tile {
    fn from(rows: [Row; 8]) -> Self {
        Self(rows)
    }
}

impl From<[Byte; 16]> for Tile {
    fn from(bytes: [Byte; 16]) -> Self {
        Self(
            bytes
                .chunks_exact(2)
                .map(|row| <[_; 2]>::try_from(row).unwrap())
                .map(Row::from)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }
}

impl Index<usize> for Tile {
    type Output = Row;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for Tile {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl IntoIterator for Tile {
    type Item = Row;

    type IntoIter = <[Row; 8] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// 8-pixel row.
///
/// This represents a single horizontal row of a [`Tile`].
#[derive(Clone, Debug)]
pub struct Row([Color; 8]);

impl Row {
    /// Horizontally flip a row.
    pub fn xflip(&mut self) {
        self.0.reverse();
    }
}

impl From<[Byte; 2]> for Row {
    fn from(data: [Byte; 2]) -> Self {
        // this will necessarily succeed
        Self(
            (0..Byte::BITS)
                // combine upper and lower bytes into colors for each bit
                .map(|bit| {
                    // Extract color bits
                    let mask = 1 << bit;
                    let bit0 = data[0] & mask != 0;
                    let bit1 = data[1] & mask != 0;
                    // Combine into color value
                    (Byte::from(bit1) << 1) | Byte::from(bit0)
                })
                // reverse the bits, since bit 7 represents the leftmost pixel,
                // and bit 0 the rightmost
                .rev()
                // convert each 2-bit value into color values
                .map(Color::from)
                // collect all row of 8 color values
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }
}

impl Index<usize> for Row {
    type Output = Color;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for Row {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl IntoIterator for Row {
    type Item = Color;
    type IntoIter = <[Color; 8] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
