use remus::bus::adapt::{Bank, View};
use remus::bus::Bus;
use remus::dev::{Device, Dynamic, Null};
use remus::{Address, Block, Shared};

use super::Mbc;

/// MBC1 cartridge type.
#[derive(Debug)]
pub struct Mbc1 {
    // Memory
    rom: Shared<Rom>,
    ram: Shared<Ram>,
}

impl Mbc1 {
    /// Constructs a new `Mbc1` with the provided configuration.
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::range_minus_one)]
    #[must_use]
    pub fn with(
        (romsz, rom): (usize, Dynamic<u16, u8>),
        (ramsz, ram): (usize, Dynamic<u16, u8>),
    ) -> Self {
        // Prepare RAM
        let ram = {
            // Determine how many banks to create
            let nbanks = ramsz / 0x4000;
            // Create banks as `View`s of the RAM
            let mut banks: Vec<Dynamic<u16, u8>> = Vec::with_capacity(nbanks);
            banks.push(Null::<u8, 0>::new().to_dynamic()); // disable RAM at index 0
            for i in 0..nbanks {
                let start = u16::try_from(0x4000 * i).unwrap();
                let end = u16::try_from(0x4000 * (i + 1) - 1).unwrap();
                banks.push(View::new(start..=end, ram.clone()).to_dynamic());
            }

            // Return RAM bank object
            Shared::new(Bank::from(&banks[..]))
        };
        // Prepare ROM
        let rom = {
            // Determine how many banks to create
            let nbanks = romsz / 0x4000;
            // Create banks as `View`s of the ROM
            let mut banks: Vec<Dynamic<u16, u8>> = Vec::with_capacity(nbanks);
            for i in 0..nbanks {
                let start = u16::try_from(0x4000 * i).unwrap();
                let end = u16::try_from(0x4000 * (i + 1) - 1).unwrap();
                banks.push(View::new(start..=end, rom.clone()).to_dynamic());
            }
            // Create the ROM bank object
            let rom0 = banks.remove(0);
            let bank = Bank::from(&banks[..]);
            let bank = Shared::new(bank);
            // Use a bus to join ROM banks together
            let mut rom = Bus::new();
            rom.map(0x0000..=0x3fff, rom0);
            rom.map(0x4000..=0x7fff, bank.clone().to_dynamic());

            Rom {
                bank,
                rom: rom.into(),
                ram: ram.clone(),
            }
        };

        Self {
            rom: Shared::new(rom),
            ram: Shared::new(ram),
        }
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
    // State
    bank: Shared<Bank<u16, u8>>,
    // Memory
    rom: Shared<Bus<u16, u8>>,
    ram: Shared<Bank<u16, u8>>,
}

impl Address<u16, u8> for Rom {
    fn read(&self, index: u16) -> u8 {
        self.rom.read(index)
    }

    #[allow(clippy::match_same_arms)]
    fn write(&mut self, index: u16, value: u8) {
        match index {
            // RAM Enable
            0x0000..=0x1fff => {
                // TODO: RAM Enable
            }
            // ROM Bank Number
            0x2000..=0x3fff => {
                // FIXME: Mask depending on ROM size
                self.bank.borrow_mut().set(match value & 0x1f {
                    0x00 => 0x00,
                    bank => bank - 1,
                } as usize);
            }
            0x4000..=0x5fff => {
                // TODO: RAM Bank Number - or - Upper Bits of ROM Bank Number
            }
            0x6000..=0x7fff => {
                // TODO: Banking Mode Select
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
type Ram = Shared<Bank<u16, u8>>;
