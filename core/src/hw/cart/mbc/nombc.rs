use remus::{Block, Device, Dynamic, Shared};

use super::Mbc;
use crate::dev::ReadOnly;

/// Rom (+ RAM) only; no MBC.
#[derive(Debug)]
pub struct NoMbc {
    rom: Shared<ReadOnly<Dynamic>>,
    ram: Dynamic,
}

impl NoMbc {
    /// Constructs a new `NoMbc` with the provided configuration.
    #[must_use]
    pub fn with(rom: Dynamic, ram: Dynamic) -> Self {
        Self {
            rom: ReadOnly::from(rom).into(),
            ram,
        }
    }
}

impl Block for NoMbc {
    fn reset(&mut self) {
        // Reset ROM
        self.rom.reset();
        // Reset RAM
        self.ram.reset();
    }
}

impl Mbc for NoMbc {
    fn rom(&self) -> Dynamic {
        self.rom.clone().to_dynamic()
    }

    fn ram(&self) -> Dynamic {
        self.ram.clone().to_dynamic()
    }
}
