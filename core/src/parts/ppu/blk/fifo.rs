use std::collections::VecDeque;

use super::meta::{Meta, Pixel, Row};

/// Pixel FIFO.
#[derive(Clone, Debug, Default)]
pub struct Fifo(VecDeque<Pixel>);

#[allow(unused)]
impl Fifo {
    /// Constructs a new `Fifo`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears the FIFO, removing all pixels.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Pushes a row of pixels into the FIFO.
    #[allow(clippy::needless_pass_by_value)]
    pub fn push(&mut self, row: Row, meta: Meta) {
        self.0.extend(
            row.into_iter()
                .skip(self.len())
                .map(|col| Pixel::new(col, meta.clone())),
        );
    }

    /// Removes the next pixel from the FIFO and returns it.
    pub fn pop(&mut self) -> Option<Pixel> {
        self.0.pop_front()
    }

    /// Returns the number of pixels in the FIFO.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the FIFO is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns `true` if the FIFO is full.
    pub fn is_full(&self) -> bool {
        self.0.len() >= 8
    }
}
