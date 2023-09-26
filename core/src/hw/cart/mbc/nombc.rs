use remus::dev::{Device, Dynamic};
use remus::{Block, Shared};

use super::Mbc;
use crate::dev::ReadOnly;

/// Rom (+ RAM) only; no MBC.
#[derive(Debug)]
pub struct NoMbc {
    // Memory
    rom: Shared<ReadOnly<Dynamic<u16, u8>>>,
    ram: Dynamic<u16, u8>,
}

impl NoMbc {
    /// Constructs a new `NoMbc` with the provided configuration.
    #[must_use]
    pub fn with(rom: Dynamic<u16, u8>, ram: Dynamic<u16, u8>) -> Self {
        Self {
            rom: ReadOnly::from(rom).into(),
            ram,
        }
    }
}

impl Block for NoMbc {
    fn reset(&mut self) {
        // Memory
        self.rom.reset();
        self.ram.reset();
    }
}

impl Mbc for NoMbc {
    fn rom(&self) -> Dynamic<u16, u8> {
        self.rom.clone().to_dynamic()
    }

    fn ram(&self) -> Dynamic<u16, u8> {
        self.ram.clone().to_dynamic()
    }
}
