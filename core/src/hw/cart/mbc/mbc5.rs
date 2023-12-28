use log::{trace, warn};
use remus::bus::adapt;
use remus::dev::{Device, Dynamic};
use remus::{Address, Block, Shared};

use super::{Mbc, Memory};
use crate::arch::Bus;

type Bank = adapt::Bank<u16, u8>;

/// [MBC5][mbc5] cartridge type.
///
/// [mbc5]: https://gbdev.io/pandocs/MBC5.html
#[derive(Debug)]
pub struct Mbc5 {
    // Memory
    rom: Shared<Rom>,
    ram: Shared<Ram>,
}

impl Mbc5 {
    /// Constructs a new `Mbc5` with the provided configuration.
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

impl Block for Mbc5 {
    fn reset(&mut self) {
        // Memory
        self.rom.reset();
        self.ram.reset();
    }
}

impl Mbc for Mbc5 {
    fn rom(&self) -> Dynamic<u16, u8> {
        self.rom.clone().to_dynamic()
    }

    fn ram(&self) -> Dynamic<u16, u8> {
        self.ram.clone().to_dynamic()
    }
}

/// MBC5 ROM.
#[allow(clippy::struct_field_names)]
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
        trace!("mbc5::Rom::write({index:#06x}, {value:#04x})");
        match index {
            // RAM Enable: value[3:0]: 0xA (enable) | 0x0 (disable)
            0x0000..=0x1fff => {
                warn!("unimplemented: Mbc5::write({index:#06x}, {value:#04x})");
                // TODO
            }
            // ROM Bank Number: bank[7:0] <-- value[7:0]
            0x2000..=0x3fff => {
                let bank = self.rom.borrow().get();
                let size = self.rom.borrow().len();
                let bits = 0x00ff & ((value as usize) % size);
                let diff = 0x00ff & (bank ^ bits);
                self.rom.borrow_mut().set(bank ^ diff);
            }
            // ROM Bank Number: bank[8] <-- value[0]
            0x4000..=0x5fff => {
                let bank = self.rom.borrow().get();
                let size = self.rom.borrow().len();
                let bits = 0x0100 & (((value as usize) << 8) % size);
                let diff = 0x0100 & (bank ^ bits);
                self.rom.borrow_mut().set(bank ^ diff);
            }
            // RAM Bank Number: bank[3:0] <-- value[3:0]
            0x6000..=0x7fff => {
                let bank = self.rom.borrow().get();
                let size = self.rom.borrow().len();
                let bits = 0x0f & ((value as usize) % size);
                let diff = 0x0f & (bank ^ bits);
                self.ram.borrow_mut().set(bank ^ diff);
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

/// MBC5 RAM.
type Ram = Bank;
