use remus::bus::adapt::{Bank, View};
use remus::bus::Bus;
use remus::dev::Null;
use remus::{Address, Block, Device, Dynamic, Shared};

use super::Mbc;

/// MBC1 cartridge type.
#[derive(Debug)]
pub struct Mbc1 {
    rom: Shared<Rom>,
    ram: Shared<Ram>,
}

impl Mbc1 {
    /// Constructs a new `Mbc1` with the provided configuration.
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn with(rom: Dynamic, ram: Dynamic, _battery: bool) -> Self {
        // Prepare RAM
        #[allow(clippy::vec_init_then_push)]
        let ram = {
            // Determine how many banks to create
            let ramsz = ram.len();
            let nbanks = ramsz / 0x4000;
            // Create banks as `View`s of the RAM
            let mut banks: Vec<Dynamic> = Vec::default();
            banks.push(Null::<0>::new().to_dynamic()); // disable RAM at index 0
            for i in 0..nbanks {
                let range = (0x4000 * i)..(0x4000 * (i + 1));
                banks.push(View::new(ram.clone(), range).to_dynamic());
            }

            // Return RAM bank object
            Shared::new(banks.into())
        };
        // Prepare ROM
        let rom = {
            // Determine how many banks to create
            let romsz = rom.len();
            let nbanks = romsz / 0x4000;
            // Create banks as `View`s of the ROM
            let mut banks: Vec<Dynamic> = Vec::default();
            for i in 0..nbanks {
                let range = (0x4000 * i)..(0x4000 * (i + 1));
                banks.push(View::new(rom.clone(), range).to_dynamic());
            }
            // Create the ROM bank object
            let rom0 = banks.remove(0);
            let bank = Bank::from(banks);
            let rom = Shared::new(bank);
            // Use a bus to join ROM banks together
            let mut bus = Bus::new();
            bus.map(0x0000, rom0);
            bus.map(0x4000, rom.clone().to_dynamic());
            let bus = Shared::new(bus);
            // Clone the RAM bank object
            let ram = ram.clone();

            Rom { bus, rom, ram }
        };

        Self {
            rom: Shared::new(rom),
            ram: Shared::new(ram),
        }
    }
}

impl Block for Mbc1 {
    fn reset(&mut self) {
        // Reset ROM
        self.rom.reset();
        // Reset RAM
        self.ram.reset();
    }
}

impl Mbc for Mbc1 {
    fn rom(&self) -> Dynamic {
        self.rom.clone().to_dynamic()
    }

    fn ram(&self) -> Dynamic {
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

impl Address<u8> for Rom {
    fn read(&self, index: usize) -> u8 {
        self.bus.read(index)
    }

    #[allow(clippy::match_same_arms)]
    fn write(&mut self, index: usize, value: u8) {
        match index {
            // RAM Enable
            0x0000..=0x1fff => {
                // TODO: RAM Enable
            }
            // ROM Bank Number
            0x2000..=0x3fff => {
                // FIXME: Mask depending on ROM size
                self.rom.borrow_mut().set(match value & 0x1f {
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
        // Reset bus
        self.bus.reset();
        // Reset ROM
        self.rom.reset();
        // Reset RAM
        self.ram.reset();
    }
}

impl Device for Rom {
    fn contains(&self, index: usize) -> bool {
        self.bus.contains(index)
    }

    fn len(&self) -> usize {
        self.bus.len()
    }
}

/// MBC1 RAM.
type Ram = Shared<Bank>;
