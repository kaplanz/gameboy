use remus::dev::{Device, Dynamic};
use remus::{Block, Shared};

use super::Mbc;
use crate::dev::ReadOnly;

/// Rom (+ RAM) only; no MBC.
#[derive(Debug)]
pub struct Raw {
    // Memory
    rom: Shared<ReadOnly<Dynamic<u16, u8>>>,
    ram: Dynamic<u16, u8>,
}

impl Raw {
    /// Constructs a new `NoMbc` with the provided configuration.
    #[must_use]
    pub fn with(rom: Dynamic<u16, u8>, ram: Dynamic<u16, u8>) -> Self {
        Self {
            rom: ReadOnly::from(rom).into(),
            ram,
        }
    }
}

impl Block for Raw {
    fn reset(&mut self) {
        // Memory
        self.rom.reset();
        self.ram.reset();
    }
}

impl Mbc for Raw {
    fn rom(&self) -> Dynamic<u16, u8> {
        self.rom.clone().to_dynamic()
    }

    fn ram(&self) -> Dynamic<u16, u8> {
        self.ram.clone().to_dynamic()
    }
}
