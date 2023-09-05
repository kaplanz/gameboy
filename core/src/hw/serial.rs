//! Serial chip.

use remus::bus::Bus;
use remus::{Block, Board, Machine};

#[derive(Debug, Default)]
pub struct Serial;

impl Block for Serial {
    fn reset(&mut self) {}
}

impl Board for Serial {
    fn connect(&self, _: &mut Bus) {}
}

impl Machine for Serial {
    fn enabled(&self) -> bool {
        todo!()
    }

    fn cycle(&mut self) {
        todo!()
    }
}
