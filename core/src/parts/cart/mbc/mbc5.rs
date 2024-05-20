use log::{debug, trace};
use remus::mem::{Error, Memory, Result};
use remus::mio::{Bus, Mmio};
use remus::reg::Register;
use remus::{Block, Byte, Shared, Word};

use super::{Data, Mbc};

/// [MBC5][mbc5] cartridge kind.
///
/// [mbc5]: https://gbdev.io/pandocs/MBC5.html
#[derive(Debug)]
pub struct Mbc5 {
    ctl: Control,
    rom: Shared<Rom>,
    ram: Shared<Ram>,
}

impl Mbc5 {
    /// Constructs a new `Mbc5`.
    #[must_use]
    pub fn new(rom: Data, ram: Data) -> Self {
        let ctl = Control::default();
        Self {
            rom: Rom::new(ctl.rom.0.clone(), ctl.rom.1.clone(), rom).into(),
            ram: Ram::new(ctl.ena.clone(), ctl.ram.clone(), ram).into(),
            ctl,
        }
    }
}

impl Block for Mbc5 {
    fn reset(&mut self) {
        self.rom.reset();
        self.ram.reset();
    }
}

impl Mbc for Mbc5 {
    fn rom(&self) -> Data {
        self.rom.borrow().mem.clone()
    }

    fn ram(&self) -> Data {
        self.ram.borrow().mem.clone()
    }
}

impl Memory for Mbc5 {
    fn read(&self, _: Word) -> Result<Byte> {
        Err(Error::Misuse)
    }

    #[allow(clippy::match_same_arms)]
    fn write(&mut self, addr: Word, data: Byte) -> Result<()> {
        trace!("Mbc5::write({addr:#06x}, {data:#04x})");
        match addr {
            // RAM Enable
            0x0000..=0x1fff => {
                // ctl.ena <- data[3:0] == 0xA
                self.ctl.ena.store(data);
            }
            // ROM Bank Number [8:0]
            0x2000..=0x2fff => {
                // ctl.rom[8:0] <- data[8:0]
                self.ctl.rom.0.store(data);
            }
            // ROM Bank Number [9]
            0x3000..=0x3fff => {
                // ctl.rom[9] <- data[0]
                self.ctl.rom.1.store(data);
            }
            // RAM Bank Number
            0x4000..=0x5fff => {
                // ctl.ram[2:0] <- data[1:0]
                self.ctl.ram.store(data);
            }
            _ => unreachable!(), // TODO: some error here
        }
        Ok(())
    }
}

impl Mmio for Mbc5 {
    fn attach(&self, bus: &mut Bus) {
        self.ctl.attach(bus);
        bus.map(0x0000..=0x7fff, self.rom.clone().into());
        bus.map(0xa000..=0xbfff, self.ram.clone().into());
    }

    fn detach(&self, bus: &mut Bus) {
        self.ctl.detach(bus);
        assert!(bus.unmap(&self.rom.clone().into()));
        assert!(bus.unmap(&self.ram.clone().into()));
    }
}

/// MBC5 registers.
///
/// | Address | Size | Name | Description           |
/// |:-------:|------|------|-----------------------|
/// | `$0000` | 1bit | ENA  | RAM Enable.           |
/// | `$2000` | 8bit | LO   | ROM Bank Number (LO). |
/// | `$3000` | 1bit | HI   | ROM Bank Number (HI). |
/// | `$4000` | 4bit | RAM  | RAM Bank Number.      |
#[rustfmt::skip]
#[derive(Clone, Debug, Default)]
struct Control {
    /// RAM Enable.
    ena: Shared<Enable>,
    /// ROM Bank Number.
    rom: (Shared<RomBankLo>, Shared<RomBankHi>),
    /// RAM Bank Number.
    ram: Shared<RamBank>,
}

impl Block for Control {
    fn reset(&mut self) {
        self.ena.take();
        self.rom.0.take();
        self.rom.1.take();
        self.ram.take();
    }
}

impl Mmio for Control {
    fn attach(&self, bus: &mut Bus) {
        bus.map(0x0000..=0x1fff, self.ena.clone().into());
        bus.map(0x2000..=0x2fff, self.rom.0.clone().into());
        bus.map(0x3000..=0x3fff, self.rom.1.clone().into());
        bus.map(0x4000..=0x5fff, self.ram.clone().into());
    }

    fn detach(&self, bus: &mut Bus) {
        assert!(bus.unmap(&self.ena.clone().into()));
        assert!(bus.unmap(&self.rom.0.clone().into()));
        assert!(bus.unmap(&self.rom.1.clone().into()));
        assert!(bus.unmap(&self.ram.clone().into()));
    }
}

/// ROM Enable.
#[derive(Debug, Default)]
struct Enable(bool);

impl Memory for Enable {
    fn read(&self, _: Word) -> Result<Byte> {
        Err(Error::Misuse)
    }

    fn write(&mut self, _: Word, data: Byte) -> Result<()> {
        self.store(data);
        Ok(())
    }
}

impl Register for Enable {
    type Value = Byte;

    fn load(&self) -> Self::Value {
        Byte::from(self.0)
    }

    fn store(&mut self, value: Self::Value) {
        self.0 = value & 0x0f == 0x0a;
        debug!("RAM Enable: {}", self.0);
    }
}

/// ROM Bank Number (bits 8:0).
#[derive(Debug, Default)]
struct RomBankLo(Byte);

impl Memory for RomBankLo {
    fn read(&self, _: Word) -> Result<Byte> {
        Err(Error::Misuse)
    }

    fn write(&mut self, _: Word, data: Byte) -> Result<()> {
        self.store(data);
        Ok(())
    }
}

impl Register for RomBankLo {
    type Value = Byte;

    fn load(&self) -> Self::Value {
        self.0
    }

    fn store(&mut self, value: Self::Value) {
        self.0 = value;
        debug!("ROM Bank Number [8:0]: {:#04x}", self.0);
    }
}

/// ROM Bank Number (bit 9).
#[derive(Debug, Default)]
struct RomBankHi(Byte);

impl Memory for RomBankHi {
    fn read(&self, _: Word) -> Result<Byte> {
        Err(Error::Misuse)
    }

    fn write(&mut self, _: Word, data: Byte) -> Result<()> {
        self.store(data);
        Ok(())
    }
}

impl Register for RomBankHi {
    type Value = Byte;

    fn load(&self) -> Self::Value {
        self.0 & 0x01
    }

    fn store(&mut self, value: Self::Value) {
        self.0 = 0x01 & value;
        debug!("ROM Bank Number [9]: {:#04x}", self.0);
    }
}

/// RAM Bank Number.
#[derive(Debug, Default)]
struct RamBank(Byte);

impl Memory for RamBank {
    fn read(&self, _: Word) -> Result<Byte> {
        Err(Error::Misuse)
    }

    fn write(&mut self, _: Word, data: Byte) -> Result<()> {
        self.store(data);
        Ok(())
    }
}

impl Register for RamBank {
    type Value = Byte;

    fn load(&self) -> Self::Value {
        self.0 & 0x0f
    }

    fn store(&mut self, value: Self::Value) {
        self.0 = 0x0f & value;
        debug!("RAM Bank Number: {:#04x}", self.0);
    }
}

/// MBC5 ROM.
#[derive(Debug)]
struct Rom {
    ilo: Shared<RomBankLo>,
    ihi: Shared<RomBankHi>,
    mem: Data,
}

impl Rom {
    /// Constructs a new `Rom`.
    fn new(ilo: Shared<RomBankLo>, ihi: Shared<RomBankHi>, mem: Data) -> Self {
        Self { ilo, ihi, mem }
    }

    /// Adjusts addresses by internal bank number.
    fn adjust(&self, addr: Word) -> usize {
        let bank = {
            let lo = usize::from(self.ilo.load());
            let hi = usize::from(self.ihi.load());
            hi << 8 | lo
        };
        let addr = usize::from(addr);
        (bank << 14 | addr & 0x3fff) % self.mem.len()
    }
}

impl Block for Rom {
    fn reset(&mut self) {
        std::mem::take(&mut self.ilo);
    }
}

impl Memory for Rom {
    fn read(&self, addr: Word) -> Result<Byte> {
        let index = match addr {
            0x0000..=0x3fff => usize::from(addr),
            0x4000..=0x7fff => self.adjust(addr),
            _ => return Err(Error::Range),
        };
        self.mem.get(index).ok_or(Error::Range).copied()
    }

    fn write(&mut self, _: Word, _: Byte) -> Result<()> {
        Err(Error::Misuse)
    }
}

/// MBC5 RAM.
#[derive(Debug)]
struct Ram {
    ena: Shared<Enable>,
    idx: Shared<RamBank>,
    mem: Data,
}

impl Ram {
    /// Constructs a new `Ram`.
    fn new(ena: Shared<Enable>, idx: Shared<RamBank>, mem: Data) -> Self {
        Self { ena, idx, mem }
    }

    /// Adjusts addresses by internal bank number.
    fn adjust(&self, addr: Word) -> usize {
        let bank = usize::from(self.idx.load());
        let addr = usize::from(addr);
        (bank << 13 | addr & 0x1fff) % self.mem.len()
    }
}

impl Block for Ram {
    fn reset(&mut self) {
        std::mem::take(&mut self.ena);
        std::mem::take(&mut self.idx);
    }
}

impl Memory for Ram {
    fn read(&self, addr: Word) -> Result<Byte> {
        // Error when disabled
        if self.ena.load() == 0 {
            return Err(Error::Busy);
        }
        // Perform adjusted read
        let index = self.adjust(addr);
        self.mem.get(index).ok_or(Error::Range).copied()
    }

    fn write(&mut self, addr: Word, data: Byte) -> Result<()> {
        // Error when disabled
        if self.ena.load() == 0 {
            return Err(Error::Busy);
        }
        // Perform adjusted write
        let index = self.adjust(addr);
        *self.mem.get_mut(index).ok_or(Error::Range)? = data;
        Ok(())
    }
}
