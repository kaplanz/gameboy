use remus::dev::{Device, Dynamic};
use remus::{Block, Shared};

use super::Mbc;
use crate::dev::ReadOnly;

/// Rom (+ RAM) only; no MBC.
#[derive(Debug)]
pub struct None {
    // Memory
    rom: Shared<ReadOnly<Dynamic<u16, u8>>>,
    ram: Dynamic<u16, u8>,
}

impl None {
    /// Constructs a new `None` with the provided configuration.
    #[must_use]
    pub fn with(rom: Dynamic<u16, u8>, ram: Dynamic<u16, u8>) -> Self {
        Self {
            rom: ReadOnly::from(rom).into(),
            ram,
        }
    }
}

impl Block for None {
    fn reset(&mut self) {
        // Memory
        self.rom.reset();
        self.ram.reset();
    }
}

impl Mbc for None {
    fn rom(&self) -> Dynamic<u16, u8> {
        self.rom.clone().to_dynamic()
    }

    fn ram(&self) -> Dynamic<u16, u8> {
        self.ram.clone().to_dynamic()
    }
}
