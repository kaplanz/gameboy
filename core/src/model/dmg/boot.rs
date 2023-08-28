//! Boot ROM.

use std::array::TryFromSliceError;

use remus::bus::adapt::Bank;
use remus::bus::Bus;
use remus::dev::Device;
use remus::reg::Register;
use remus::{mem, Address, Block, Board, Cell, Shared};
use thiserror::Error;

use crate::dev::ReadOnly;

/// Boot ROM.
///
/// Implements [`Device`] to allowing mapping onto a [`Bus`]. To support
/// boot ROM disable, the [`disable`](Rom::disable) device must be mapped
/// separately.
#[derive(Debug, Default)]
pub struct Rom {
    // State
    rom: Shared<mem::Rom<0x100>>,
    // Devices
    // ┌────────┬──────┬─────┐
    // │  Size  │ Name │ Dev │
    // ├────────┼──────┼─────┤
    // │  8 KiB │ Boot │ ROM │
    // └────────┴──────┴─────┘
    bank: Shared<Bank>,
    // Control
    // ┌──────┬──────────┬─────┐
    // │ Size │   Name   │ Dev │
    // ├──────┼──────────┼─────┤
    // │  1 B │ Disable  │ Reg │
    // └──────┴──────────┴─────┘
    disable: Shared<Disable>,
}

impl Rom {
    /// Gets a read-only reference to the boot ROM.
    #[must_use]
    pub fn rom(&self) -> ReadOnly<impl Device> {
        ReadOnly::from(self.bank.clone())
    }

    /// Gets a reference to the boot ROM's disable register.
    #[must_use]
    pub fn disable(&self) -> Shared<Disable> {
        self.disable.clone()
    }
}

impl Address<u8> for Rom {
    fn read(&self, index: usize) -> u8 {
        self.bank.read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        self.bank.write(index, value);
    }
}

impl Block for Rom {
    fn reset(&mut self) {
        // Reset controller
        self.disable.reset();
        self.disable.borrow_mut().bank = self.bank.clone();
        // Reset bank
        self.bank.reset();
        self.bank.borrow_mut().add(self.rom.clone().to_dynamic());
        self.bank.borrow_mut().set(0);
    }
}

impl Board for Rom {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let boot = self.rom().to_dynamic();
        let disable = self.disable().to_dynamic();

        // Map devices on bus     // ┌──────┬────────┬──────────┬─────┐
                                  // │ Addr │  Size  │   Name   │ Dev │
                                  // ├──────┼────────┼──────────┼─────┤
        bus.map(0x0000, boot);    // │ 0000 │  8 KiB │ Boot     │ ROM │
        bus.map(0xff50, disable); // │ ff50 │    1 B │ Disable  │ Reg │
                                  // └──────┴────────┴──────────┴─────┘
    }
}

impl Device for Rom {
    fn contains(&self, index: usize) -> bool {
        self.bank.contains(index)
    }

    fn len(&self) -> usize {
        self.bank.len()
    }
}

impl From<&[u8; 0x100]> for Rom {
    fn from(rom: &[u8; 0x100]) -> Self {
        let rom = mem::Rom::from(rom).into();
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
    bank: Shared<Bank>,
}

impl Address<u8> for Disable {
    fn read(&self, _: usize) -> u8 {
        self.load()
    }

    fn write(&mut self, _: usize, value: u8) {
        self.store(value);
    }
}

impl Block for Disable {
    fn reset(&mut self) {
        // Reset controller
        self.boff.reset();
        // Reset bank
        self.bank.borrow_mut().set(self.boff.load() as usize);
    }
}

impl Cell<u8> for Disable {
    fn load(&self) -> u8 {
        self.boff.load()
    }

    fn store(&mut self, value: u8) {
        self.boff.store(value);
        self.bank.borrow_mut().set(value as usize);
    }
}

impl Device for Disable {
    fn contains(&self, index: usize) -> bool {
        self.boff.contains(index)
    }

    fn len(&self) -> usize {
        self.boff.len()
    }
}

/// A type specifying general categories of [`Rom`] error.
#[derive(Debug, Error)]
pub enum Error {
    /// Body is missing, caused by truncation when attempting to construct.
    #[error("missing body")]
    Missing,
    /// Pass-through for [`TryFromSliceError`]s.
    #[error(transparent)]
    TryFromSlice(#[from] TryFromSliceError),
}
