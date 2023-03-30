use std::collections::VecDeque;

use super::pixel::{Meta, Pixel};
use super::tile::Row;

#[derive(Debug, Default)]
pub struct Fifo(VecDeque<Pixel>);

impl Fifo {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn try_append(&mut self, row: Row, meta: Meta) -> Result<(), (Row, Meta)> {
        if self.0.is_empty() {
            self.0.extend(row.iter().map(|col| Pixel::new(*col, meta)));
            Ok(())
        } else {
            Err((row, meta))
        }
    }

    pub fn pop(&mut self) -> Option<Pixel> {
        self.0.pop_front()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
