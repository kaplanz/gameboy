//! Boot ROM.

use std::array::TryFromSliceError;
use std::cell::RefCell;
use std::rc::Rc;

use remus::bus::adapt::Bank;
use remus::bus::Bus;
use remus::dev::Device;
use remus::reg::Register;
use remus::{mem, Block, Board, SharedDevice};
use thiserror::Error;

/// Boot ROM management [`Device`](Device).
#[derive(Debug, Default)]
pub struct Rom {
    // State
    rom: Rc<RefCell<mem::Rom<0x100>>>,
    // Devices
    // ┌────────┬──────┬─────┬───────┐
    // │  Size  │ Name │ Dev │ Alias │
    // ├────────┼──────┼─────┼───────┤
    // │  8 KiB │ Boot │ ROM │       │
    // └────────┴──────┴─────┴───────┘
    bank: Rc<RefCell<Bank>>,
    // Control
    // ┌──────┬──────────┬─────┬───────┐
    // │ Size │   Name   │ Dev │ Alias │
    // ├──────┼──────────┼─────┼───────┤
    // │  1 B │ Disable  │ Reg │       │
    // └──────┴──────────┴─────┴───────┘
    disable: Rc<RefCell<Disable>>,
}

impl Rom {
    /// Gets a reference to the boot ROM.
    #[must_use]
    pub fn rom(&self) -> SharedDevice {
        self.bank.clone()
    }

    /// Gets a reference to the boot ROM's disable register.
    #[must_use]
    pub fn disable(&self) -> SharedDevice {
        self.disable.clone()
    }
}

impl Block for Rom {
    fn reset(&mut self) {
        // Reset controller
        self.disable.borrow_mut().reset();
        self.disable.borrow_mut().bank = self.bank.clone();
        // Reset bank
        self.bank.borrow_mut().reset();
        self.bank.borrow_mut().add(self.rom.clone());
        self.bank.borrow_mut().set(0);
    }
}

impl Board for Rom {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let boot = self.rom();
        let disable = self.disable();

        // Map devices on bus     // ┌──────┬────────┬──────────┬─────┐
                                  // │ Addr │  Size  │   Name   │ Dev │
                                  // ├──────┼────────┼──────────┼─────┤
        bus.map(0x0000, boot);    // │ 0000 │  8 KiB │ Boot     │ ROM │
        bus.map(0xff50, disable); // │ ff50 │    1 B │ Disable  │ Reg │
                                  // └──────┴────────┴──────────┴─────┘
    }
}

impl From<&[u8; 0x100]> for Rom {
    fn from(rom: &[u8; 0x100]) -> Self {
        let rom = Rc::new(RefCell::new(mem::Rom::from(rom)));
        Self {
            rom,
            ..Default::default()
        }
    }
}

impl TryFrom<&[u8]> for Rom {
    type Error = Error;

    fn try_from(rom: &[u8]) -> Result<Self, Self::Error> {
        let rom: Result<&[u8; 0x100], _> = rom
            .get(0x000..0x100)
            .ok_or(Error::Missing)?
            .try_into()
            .map_err(Error::TryFromSlice);
        rom.map(Into::into)
    }
}

/// Boot ROM disable [`Register`](Disable).
#[derive(Debug, Default)]
pub struct Disable {
    boff: Register<u8>,
    bank: Rc<RefCell<Bank>>,
}

impl Block for Disable {
    fn reset(&mut self) {
        // Reset controller
        self.boff.reset();
        // Reset bank
        self.bank.borrow_mut().set(*self.boff as usize);
    }
}

impl Device for Disable {
    fn contains(&self, index: usize) -> bool {
        self.boff.contains(index)
    }

    fn len(&self) -> usize {
        self.boff.len()
    }

    fn read(&self, index: usize) -> u8 {
        self.boff.read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        self.boff.write(index, value);
        self.bank.borrow_mut().set(value as usize);
    }
}

/// A type specifying general categories of [`Rom`] error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("missing body")]
    Missing,
    #[error(transparent)]
    TryFromSlice(#[from] TryFromSliceError),
}
