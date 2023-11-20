//! Programmable interrupt controller.

use std::fmt::Display;

use enuf::Enuf;
use log::trace;
use remus::bus::Mux;
use remus::dev::Device;
use remus::{reg, Address, Block, Board, Cell, Location, Shared};
use thiserror::Error;

use crate::arch::Bus;

#[allow(clippy::doc_markdown)]
/// 8-bit serial control register set.
///
/// | Bit | Name     |
/// |-----|----------|
/// |  0  | VBlank   |
/// |  1  | LCD STAT |
/// |  2  | Timer    |
/// |  3  | Serial   |
/// |  4  | Joypad   |
#[derive(Clone, Copy, Debug)]
pub enum Control {
    /// `0xFF0F`: Interrupt flag.
    If,
    /// `0xFFFF`: Interrupt enable.
    Ie,
}

/// Programmable interrupt controller model.
#[derive(Debug, Default)]
pub struct Pic {
    // Control
    // ┌──────┬──────────┬─────┐
    // │ Size │   Name   │ Dev │
    // ├──────┼──────────┼─────┤
    // │  2 B │ Control  │ Reg │
    // └──────┴──────────┴─────┘
    file: File,
}

impl Pic {
    /// Fetches the first pending interrupt.
    #[must_use]
    pub fn int(&self) -> Option<Interrupt> {
        let fl = self.file.fl.load();
        let en = self.file.en.load();
        let int = (fl & en).try_into().ok();
        if let Some(int) = int {
            trace!("interrupt pending: {int:?}");
        }
        int
    }

    /// Requests an interrupt.
    pub fn req(&mut self, int: Interrupt) {
        let fl = self.file.fl.load() | (int as u8);
        self.file.fl.store(fl);
        trace!("interrupt requested: {int:?}");
    }

    /// Acknowledges an interrupt.
    pub fn ack(&mut self, int: Interrupt) {
        let fl = self.file.fl.load() & !(int as u8);
        self.file.fl.store(fl);
        trace!("interrupt acknowledged: {int:?}");
    }
}

impl Block for Pic {
    fn reset(&mut self) {
        // Control
        self.file.reset();
    }
}

impl Board<u16, u8> for Pic {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Connect boards
        self.file.connect(bus);
    }
}

impl Location<u8> for Pic {
    type Register = Control;

    fn load(&self, reg: Self::Register) -> u8 {
        match reg {
            Control::If => self.file.fl.load(),
            Control::Ie => self.file.en.load(),
        }
    }

    fn store(&mut self, reg: Self::Register, value: u8) {
        match reg {
            Control::If => self.file.fl.store(value),
            Control::Ie => self.file.en.store(value),
        }
    }
}

/// Control registers.
#[derive(Debug, Default)]
struct File {
    // ┌──────┬────────┬─────┬───────┐
    // │ Size │  Name  │ Dev │ Alias │
    // ├──────┼────────┼─────┼───────┤
    // │  1 B │ Flag   │ Reg │ IF    │
    // │  1 B │ Enable │ Reg │ IE    │
    // └──────┴────────┴─────┴───────┘
    fl: Shared<Register>,
    en: Shared<Register>,
}

impl Block for File {
    fn reset(&mut self) {
        self.fl.reset();
        self.en.reset();
    }
}

impl Board<u16, u8> for File {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let fl = self.fl.clone().to_dynamic();
        let en = self.en.clone().to_dynamic();

        // Map devices on bus         // ┌──────┬──────┬────────┬─────┐
                                      // │ Addr │ Size │  Name  │ Dev │
                                      // ├──────┼──────┼────────┼─────┤
        bus.map(0xff0f..=0xff0f, fl); // │ ff0f │  1 B │ Active │ Reg │
        bus.map(0xffff..=0xffff, en); // │ ffff │  1 B │ Enable │ Reg │
                                      // └──────┴──────┴────────┴─────┘
    }
}

/// Interrupt register.
#[derive(Debug, Default)]
pub struct Register(reg::Register<u8>);

impl Address<u16, u8> for Register {
    fn read(&self, _: u16) -> u8 {
        self.load()
    }

    fn write(&mut self, _: u16, value: u8) {
        self.store(value);
    }
}

impl Block for Register {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Cell<u8> for Register {
    fn load(&self) -> u8 {
        self.0.load() | 0xe0
    }

    fn store(&mut self, value: u8) {
        self.0.store(value & 0x1f);
    }
}

impl Device<u16, u8> for Register {}

/// Interrupt type.
#[rustfmt::skip]
#[derive(Copy, Clone, Debug)]
pub enum Interrupt {
    // ┌─────┬──────────┬─────────┐
    // │ Bit │  Source  │ Handler │
    // ├─────┼──────────┼─────────┤
    // │   0 │ VBlank   │  0x0040 │
    // │   1 │ LCD STAT │  0x0048 │
    // │   2 │ Timer    │  0x0050 │
    // │   3 │ Serial   │  0x0058 │
    // │   4 │ Joypad   │  0x0060 │
    // └─────┴──────────┴─────────┘
    VBlank  = 0b0000_0001,
    LcdStat = 0b0000_0010,
    Timer   = 0b0000_0100,
    Serial  = 0b0000_1000,
    Joypad  = 0b0001_0000,
}

impl Interrupt {
    #[rustfmt::skip]
    #[must_use]
    pub fn handler(self) -> u8 {
        match self {
            Self::VBlank  => 0x40,
            Self::LcdStat => 0x48,
            Self::Timer   => 0x50,
            Self::Serial  => 0x58,
            Self::Joypad  => 0x60,
        }
    }

    #[rustfmt::skip]
    #[must_use]
    pub fn repr(self) -> &'static str {
        match self {
            Self::VBlank  => "INT 40H",
            Self::LcdStat => "INT 48H",
            Self::Timer   => "INT 50H",
            Self::Serial  => "INT 58H",
            Self::Joypad  => "INT 60H",
        }
    }
}

impl Enuf for Interrupt {}

impl Display for Interrupt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.repr().fmt(f)
    }
}

impl TryFrom<u8> for Interrupt {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value.trailing_zeros() {
            0 => Ok(Self::VBlank),
            1 => Ok(Self::LcdStat),
            2 => Ok(Self::Timer),
            3 => Ok(Self::Serial),
            4 => Ok(Self::Joypad),
            _ => Err(Error::Unknown),
        }
    }
}

impl From<Interrupt> for u8 {
    fn from(value: Interrupt) -> Self {
        value as u8
    }
}

/// A type specifying categories of [`Pic`] error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown interrupt")]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    #[test]
    fn interrupt_u8_from_works() {
        assert_eq!(u8::from(Interrupt::VBlank),  0b0000_0001);
        assert_eq!(u8::from(Interrupt::LcdStat), 0b0000_0010);
        assert_eq!(u8::from(Interrupt::Timer),   0b0000_0100);
        assert_eq!(u8::from(Interrupt::Serial),  0b0000_1000);
        assert_eq!(u8::from(Interrupt::Joypad),  0b0001_0000);
    }

    #[rustfmt::skip]
    #[test]
    fn interrupt_try_from_u8_works() {
        assert!(matches!(Interrupt::try_from(0b0000_0001), Ok(Interrupt::VBlank)));
        assert!(matches!(Interrupt::try_from(0b0000_0010), Ok(Interrupt::LcdStat)));
        assert!(matches!(Interrupt::try_from(0b0000_0100), Ok(Interrupt::Timer)));
        assert!(matches!(Interrupt::try_from(0b0000_1000), Ok(Interrupt::Serial)));
        assert!(matches!(Interrupt::try_from(0b0001_0000), Ok(Interrupt::Joypad)));
    }
}
