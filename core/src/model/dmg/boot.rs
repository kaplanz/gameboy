//! Boot ROM.

use std::array::TryFromSliceError;

use remus::bus::adapt::Bank;
use remus::bus::Bus;
use remus::dev::Device;
use remus::{mem, Address, Block, Board, Cell, Shared};
use thiserror::Error;

use crate::dev::ReadOnly;

/// Boot ROM.
#[derive(Debug, Default)]
pub struct Rom {
    // Control
    // ┌──────┬──────────┬─────┐
    // │ Size │   Name   │ Dev │
    // ├──────┼──────────┼─────┤
    // │  1 B │ Control  │ Reg │
    // └──────┴──────────┴─────┘
    ctrl: Shared<Control>,
    // Memory
    // ┌────────┬──────┬─────┐
    // │  Size  │ Name │ Dev │
    // ├────────┼──────┼─────┤
    // │  8 KiB │ Boot │ ROM │
    // └────────┴──────┴─────┘
    bank: Shared<Bank<u16, u8>>,
}

impl Rom {
    /// Constructs a new `Rom`.
    #[must_use]
    pub fn new(rom: mem::Rom<u8, 0x100>) -> Self {
        // Construct shared blocks
        let bank = Shared::from(Bank::from([rom.to_dynamic()].as_slice()));
        let ctrl = Shared::from(Control::new(bank.clone()));
        // Construct self
        Self { ctrl, bank }
    }

    /// Gets a read-only reference to the boot ROM.
    #[must_use]
    pub fn rom(&self) -> ReadOnly<impl Device<u16, u8>> {
        ReadOnly::from(self.bank.clone())
    }

    /// Gets a reference to the boot ROM's control register.
    #[must_use]
    pub fn ctrl(&self) -> Shared<Control> {
        self.ctrl.clone()
    }
}

impl Address<u16, u8> for Rom {
    fn read(&self, index: u16) -> u8 {
        self.bank.read(index)
    }

    fn write(&mut self, index: u16, value: u8) {
        self.bank.write(index, value);
    }
}

impl Block for Rom {
    fn reset(&mut self) {
        // Control
        self.ctrl.reset();
        // Memory
        self.bank.reset();
    }
}

impl Board<u16, u8> for Rom {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus<u16, u8>) {
        // Extract devices
        let boot = self.rom().to_dynamic();
        let ctrl = self.ctrl().to_dynamic();

        // Map devices on bus           // ┌──────┬────────┬──────────┬─────┐
                                        // │ Addr │  Size  │   Name   │ Dev │
                                        // ├──────┼────────┼──────────┼─────┤
        bus.map(0x0000..=0x00ff, boot); // │ 0000 │  256 B │ Boot     │ ROM │
        bus.map(0xff50..=0xff50, ctrl); // │ ff50 │    1 B │ Control  │ Reg │
                                        // └──────┴────────┴──────────┴─────┘
    }
}

impl Device<u16, u8> for Rom {}

impl From<&[u8; 0x100]> for Rom {
    fn from(rom: &[u8; 0x100]) -> Self {
        Self::new(rom.into())
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
        rom.map(Into::into).map(Self::new)
    }
}

/// Boot ROM [`Control`].
#[derive(Debug, Default)]
pub struct Control {
    // Shared
    bank: Shared<Bank<u16, u8>>,
}

impl Control {
    /// Constructs a new `Control`.
    pub fn new(rom: Shared<Bank<u16, u8>>) -> Self {
        Self { bank: rom }
    }
}

impl Address<u16, u8> for Control {
    fn read(&self, _: u16) -> u8 {
        self.load()
    }

    fn write(&mut self, _: u16, value: u8) {
        self.store(value);
    }
}

impl Block for Control {
    fn reset(&mut self) {
        self.bank.borrow_mut().set(0);
    }
}

impl Cell<u8> for Control {
    fn load(&self) -> u8 {
        0xff
    }

    fn store(&mut self, _: u8) {
        self.bank.borrow_mut().set(1);
    }
}

impl Device<u16, u8> for Control {}

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
