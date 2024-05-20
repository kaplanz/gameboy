use remus::mio::{Bus, Mmio};
use remus::{Block, Shared};

use super::{Data, Mbc};

/// Cartridge ROM.
type Rom = remus::mem::Rom<Data>;
/// Cartridge RAM.
type Ram = remus::mem::Ram<Data>;

/// Bare ROM + RAM.
#[derive(Debug)]
pub struct Bare {
    rom: Shared<Rom>,
    ram: Shared<Ram>,
}

impl Bare {
    /// Constructs a new `Bare`.
    #[must_use]
    pub fn new(rom: Data, ram: Data) -> Self {
        Self {
            rom: Rom::from(rom).into(),
            ram: Ram::from(ram).into(),
        }
    }
}

impl Block for Bare {}

impl Mbc for Bare {
    fn rom(&self) -> Data {
        self.rom.borrow().inner().clone()
    }

    fn ram(&self) -> Data {
        self.ram.borrow().inner().clone()
    }
}

impl Mmio for Bare {
    fn attach(&self, bus: &mut Bus) {
        bus.map(0x0000..=0x7fff, self.rom.clone().into());
        bus.map(0xa000..=0xbfff, self.ram.clone().into());
    }

    fn detach(&self, bus: &mut Bus) {
        assert!(bus.unmap(&self.rom.clone().into()));
        assert!(bus.unmap(&self.ram.clone().into()));
    }
}
