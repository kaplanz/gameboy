//! Audio chip.

use remus::bus::Bus;
use remus::dev::Null;
use remus::{Block, Device, Machine};

use crate::dmg::Board;

#[derive(Debug, Default)]
pub struct Audio;

impl Block for Audio {
    fn reset(&mut self) {}
}

impl Board for Audio {
    fn connect(&self, bus: &mut Bus) {
        let audio = Null::<0x17>::new().to_shared();
        let wave = Null::<0x10>::new().to_shared();
        bus.map(0xff10, audio);
        bus.map(0xff30, wave);
    }
}

impl Machine for Audio {
    fn enabled(&self) -> bool {
        todo!()
    }

    fn cycle(&mut self) {
        todo!()
    }
}
