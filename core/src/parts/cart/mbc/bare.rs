use std::io;

use remus::mio::Device;
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
    fn rom(&self) -> Device {
        self.rom.clone().into()
    }

    fn ram(&self) -> Device {
        self.ram.clone().into()
    }

    fn flash(&mut self, buf: &mut impl io::Read) -> io::Result<usize> {
        buf.read(self.ram.borrow_mut().inner_mut())
    }

    fn dump(&self, buf: &mut impl io::Write) -> io::Result<usize> {
        buf.write(self.ram.borrow().inner())
    }
}
