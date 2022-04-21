use std::cell::RefCell;
use std::rc::Rc;

use remus::{Block, SharedDevice};

use super::Mbc;
use crate::dev::ReadOnly;

/// Rom (+ RAM) only; no MBC.
#[derive(Debug)]
pub struct NoMbc {
    rom: Rc<RefCell<ReadOnly>>,
    ram: SharedDevice,
}

impl NoMbc {
    /// Constructs a new `NoMbc` with the provided configuration.
    pub fn with(rom: SharedDevice, ram: SharedDevice) -> Self {
        Self {
            rom: Rc::new(RefCell::new(ReadOnly::from(rom))),
            ram,
        }
    }
}

impl Block for NoMbc {
    fn reset(&mut self) {
        // Reset ROM
        self.rom.borrow_mut().reset();
        // Reset RAM
        self.ram.borrow_mut().reset();
    }
}

impl Mbc for NoMbc {
    fn rom(&self) -> SharedDevice {
        self.rom.clone()
    }

    fn ram(&self) -> SharedDevice {
        self.ram.clone()
    }
}
