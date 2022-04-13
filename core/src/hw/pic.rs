use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use remus::reg::Register;
use remus::Block;
use thiserror::Error;

use crate::util::Bitflags;

#[derive(Debug, Default)]
pub struct Pic {
    // Interrupt Enable (IE)
    pub enable: Rc<RefCell<Register<u8>>>,
    // Interrupt Flag (IF)
    pub active: Rc<RefCell<Register<u8>>>,
}

impl Pic {
    pub fn int(&self) -> Option<Interrupt> {
        let active = **self.active.borrow();
        let enable = **self.enable.borrow();
        (active & enable).try_into().ok()
    }

    pub fn req(&mut self, int: Interrupt) {
        **self.active.borrow_mut() |= int as u8;
    }

    pub fn ack(&mut self, int: Interrupt) {
        **self.active.borrow_mut() &= !(int as u8);
    }
}

impl Block for Pic {
    fn reset(&mut self) {
        // Reset registers
        self.enable.borrow_mut().reset();
        self.active.borrow_mut().reset();
    }
}

#[rustfmt::skip]
#[derive(Copy, Clone, Debug)]
pub enum Interrupt {
    // ┌─────┬──────────┬─────────┐
    // │ BIT │  SOURCE  │ HANDLER │
    // ├─────┼──────────┼─────────┤
    // │   0 │   VBlank │  0x0040 │
    // │   1 │ LCD STAT │  0x0048 │
    // │   2 │    Timer │  0x0050 │
    // │   3 │   Serial │  0x0058 │
    // │   4 │   Joypad │  0x0060 │
    // └─────┴──────────┴─────────┘
    VBlank  = 0b00000001,
    LcdStat = 0b00000010,
    Timer   = 0b00000100,
    Serial  = 0b00001000,
    Joypad  = 0b00010000,
}

impl Interrupt {
    #[rustfmt::skip]
    pub fn handler(&self) -> u8 {
        match self {
            Self::VBlank  => 0x40,
            Self::LcdStat => 0x48,
            Self::Timer   => 0x50,
            Self::Serial  => 0x58,
            Self::Joypad  => 0x60,
        }
    }

    #[rustfmt::skip]
    pub fn repr(&self) -> &'static str {
        match self {
            Self::VBlank  => "INT 40H",
            Self::LcdStat => "INT 48H",
            Self::Timer   => "INT 50H",
            Self::Serial  => "INT 58H",
            Self::Joypad  => "INT 60H",
        }
    }
}

impl Bitflags for Interrupt {}

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
        assert_eq!(u8::from(Interrupt::VBlank),  0b00000001);
        assert_eq!(u8::from(Interrupt::LcdStat), 0b00000010);
        assert_eq!(u8::from(Interrupt::Timer),   0b00000100);
        assert_eq!(u8::from(Interrupt::Serial),  0b00001000);
        assert_eq!(u8::from(Interrupt::Joypad),  0b00010000);
    }

    #[rustfmt::skip]
    #[test]
    fn interrupt_try_from_u8_works() {
        assert!(matches!(Interrupt::try_from(0b00000001), Ok(Interrupt::VBlank)));
        assert!(matches!(Interrupt::try_from(0b00000010), Ok(Interrupt::LcdStat)));
        assert!(matches!(Interrupt::try_from(0b00000100), Ok(Interrupt::Timer)));
        assert!(matches!(Interrupt::try_from(0b00001000), Ok(Interrupt::Serial)));
        assert!(matches!(Interrupt::try_from(0b00010000), Ok(Interrupt::Joypad)));
    }
}
