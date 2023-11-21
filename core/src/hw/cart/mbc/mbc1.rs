use log::warn;
use remus::bus::adapt;
use remus::dev::{Device, Dynamic};
use remus::{Address, Block, Shared};

use super::{Mbc, Memory};
use crate::arch::Bus;

type Bank = adapt::Bank<u16, u8>;

/// [MBC1][mbc1] cartridge type.
///
/// [mbc1]: https://gbdev.io/pandocs/MBC1.html
#[derive(Debug)]
pub struct Mbc1 {
    // Memory
    rom: Shared<Rom>,
    ram: Shared<Ram>,
}

impl Mbc1 {
    /// Constructs a new `Mbc1` with the provided configuration.
    #[must_use]
    pub(in super::super) fn with(rom: Memory, ram: Memory) -> Self {
        // Prepare RAM
        let ram = Bank::from(&*ram.fracture(0x2000)).to_shared();
        // Prepare ROM
        let rom = {
            // Create views
            let view = rom.fracture(0x4000);
            let root = view[0].clone();
            let bank = Bank::from(&*view).to_shared();
            // Set default bank
            bank.borrow_mut().set(0b1);
            // Create memory map
            let mmap = Bus::from([
                (0x0000..=0x3fff, root),
                (0x4000..=0x7fff, bank.clone().to_dynamic()),
            ])
            .to_shared();
            // Create ROM
            Rom {
                bus: mmap,
                rom: bank,
                ram: ram.clone(),
            }
        }
        .to_shared();

        Self { rom, ram }
    }
}

impl Block for Mbc1 {
    fn reset(&mut self) {
        // Memory
        self.rom.reset();
        self.ram.reset();
    }
}

impl Mbc for Mbc1 {
    fn rom(&self) -> Dynamic<u16, u8> {
        self.rom.clone().to_dynamic()
    }

    fn ram(&self) -> Dynamic<u16, u8> {
        self.ram.clone().to_dynamic()
    }
}

/// MBC1 ROM.
#[derive(Debug)]
struct Rom {
    bus: Shared<Bus>,
    rom: Shared<Bank>,
    ram: Shared<Bank>,
}

impl Address<u16, u8> for Rom {
    fn read(&self, index: u16) -> u8 {
        self.bus.read(index)
    }

    #[allow(clippy::match_same_arms)]
    fn write(&mut self, index: u16, value: u8) {
        match index {
            // RAM Enable: value[3:0]: 0xA (enable) | 0x0 (disable)
            0x0000..=0x1fff => {
                warn!("unimplemented: Mbc1::write({index:#06x}, {value:#04x})");
                // TODO
            }
            // ROM Bank Number: bank[4:0] <-- value[4:0]
            #[rustfmt::skip]
            0x2000..=0x3fff => {
                let bank = self.rom.borrow().get();
                let bits = 0x001f & (value as usize) | usize::from(value != 0);
                let diff = 0x001f & (bank ^ bits);  // ^^^ ensure non-zero ^^^
                self.rom.borrow_mut().set(bank ^ diff);
            }
            // ROM Bank Number: TODO
            0x4000..=0x5fff => {
                warn!("unimplemented: Mbc1::write({index:#06x}, {value:#04x})");
                // TODO
            }
            // Banking Mode Select: TODO
            0x6000..=0x7fff => {
                warn!("unimplemented: Mbc1::write({index:#06x}, {value:#04x})");
                // TODO
            }
            _ => panic!(), // TODO: some error here
        }
    }
}

impl Block for Rom {
    fn reset(&mut self) {
        // Memory
        self.rom.reset();
        self.ram.reset();
    }
}

impl Device<u16, u8> for Rom {}

/// MBC1 RAM.
type Ram = Bank;
