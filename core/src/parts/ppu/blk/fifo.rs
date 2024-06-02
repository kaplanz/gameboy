use std::collections::VecDeque;

use super::pixel::{Meta, Pixel};
use super::tile::Row;

/// Pixel FIFO.
#[derive(Clone, Debug, Default)]
pub struct Fifo(VecDeque<Pixel>);

impl Fifo {
    /// Clears the FIFO, removing all pixels.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Pushes a row of pixels into the FIFO.
    ///
    /// # Errors
    ///
    /// Returns
    pub fn push(&mut self, row: &Row, meta: Meta) -> bool {
        if self.is_empty() {
            self.0.extend(row.iter().map(|col| Pixel::new(*col, meta)));
            true
        } else {
            false
        }
    }

    /// Removes the next pixel from the FIFO and returns it.
    pub fn pop(&mut self) -> Option<Pixel> {
        self.0.pop_front()
    }

    /// Returns the number of pixels in the FIFO.
    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the FIFO is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
