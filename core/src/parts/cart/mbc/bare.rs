use std::io;

use rugby_arch::mio::Device;
use rugby_arch::{Block, Shared};

use super::{Data, Mbc};

/// Cartridge ROM.
type Rom = rugby_arch::mem::Rom<Data>;
/// Cartridge RAM.
type Ram = rugby_arch::mem::Ram<Data>;

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
            rom: Shared::new(Rom::from(rom)),
            ram: Shared::new(Ram::from(ram)),
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
