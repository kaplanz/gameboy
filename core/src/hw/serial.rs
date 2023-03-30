//! Serial chip.

use remus::bus::Bus;
use remus::dev::Null;
use remus::{Block, Device, Machine};

use crate::dmg::Board;

#[derive(Debug, Default)]
pub struct Serial;

impl Block for Serial {
    fn reset(&mut self) {}
}

impl Board for Serial {
    fn connect(&self, bus: &mut Bus) {
        let null = Null::<0x2>::new().to_shared();
        bus.map(0xff01, null);
    }
}

impl Machine for Serial {
    fn enabled(&self) -> bool {
        todo!()
    }

    fn cycle(&mut self) {
        todo!()
    }
}