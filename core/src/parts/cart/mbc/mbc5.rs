use std::io;

use log::{debug, trace};
use remus::mem::{Error, Memory, Result};
use remus::mio::Device;
use remus::reg::Register;
use remus::{Block, Byte, Shared, Word};

use super::{Data, Mbc};

/// [MBC5][mbc5] cartridge type.
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
            rom: Rom::new(ctl.clone(), rom).into(),
            ram: Ram::new(ctl.clone(), ram).into(),
            ctl,
        }
    }
}

impl Block for Mbc5 {
    fn reset(&mut self) {
        self.ctl.reset();
    }
}

impl Mbc for Mbc5 {
    fn rom(&self) -> Device {
        self.rom.clone().into()
    }

    fn ram(&self) -> Device {
        self.ram.clone().into()
    }

    fn flash(&mut self, buf: &mut impl io::Read) -> io::Result<usize> {
        buf.read(&mut self.ram.borrow_mut().mem)
    }

    fn dump(&self, buf: &mut impl io::Write) -> io::Result<usize> {
        buf.write(&self.ram.borrow().mem)
    }
}

/// MBC5 registers.
///
/// |     Address     | Size | Name | Description           |
/// |:---------------:|------|------|-----------------------|
/// | `$0000..=$1FFF` | 1bit | ENA  | RAM Enable.           |
/// | `$2000..=$2FFF` | 8bit | LO   | ROM Bank Number (LO). |
/// | `$3000..=$3FFF` | 1bit | HI   | ROM Bank Number (HI). |
/// | `$4000..=$7FFF` | 4bit | RAM  | RAM Bank Number.      |
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
    ctl: Control,
    mem: Data,
}

impl Rom {
    /// Constructs a new `Rom`.
    fn new(ctl: Control, mem: Data) -> Self {
        Self { ctl, mem }
    }

    /// Adjusts addresses by internal bank number.
    fn adjust(&self, addr: Word) -> usize {
        let bank = {
            let lo = usize::from(self.ctl.rom.0.load());
            let hi = usize::from(self.ctl.rom.1.load());
            hi << 8 | lo
        };
        let addr = usize::from(addr);
        (bank << 14 | addr & 0x3fff) % self.mem.len().max(0x8000)
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
            _ => return Err(Error::Range),
        }
        Ok(())
    }
}

/// MBC5 RAM.
#[derive(Debug)]
struct Ram {
    ctl: Control,
    mem: Data,
}

impl Ram {
    /// Constructs a new `Ram`.
    fn new(ctl: Control, mem: Data) -> Self {
        Self { ctl, mem }
    }

    /// Adjusts addresses by internal bank number.
    fn adjust(&self, addr: Word) -> usize {
        let bank = usize::from(self.ctl.ram.load());
        let addr = usize::from(addr);
        (bank << 13 | addr & 0x1fff) % self.mem.len().max(0x2000)
    }
}

impl Memory for Ram {
    fn read(&self, addr: Word) -> Result<Byte> {
        // Error when disabled
        if self.ctl.ena.load() == 0 {
            return Err(Error::Busy);
        }
        // Perform adjusted read
        let index = self.adjust(addr);
        self.mem.get(index).ok_or(Error::Range).copied()
    }

    fn write(&mut self, addr: Word, data: Byte) -> Result<()> {
        // Error when disabled
        if self.ctl.ena.load() == 0 {
            return Err(Error::Busy);
        }
        // Perform adjusted write
        let index = self.adjust(addr);
        *self.mem.get_mut(index).ok_or(Error::Range)? = data;
        Ok(())
    }
}
