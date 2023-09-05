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
    bank: Shared<Bank>,
}

impl Rom {
    /// Constructs a new `Rom`.
    #[must_use]
    pub fn new(rom: mem::Rom<0x100>) -> Self {
        // Construct shared blocks
        let bank = Shared::from(Bank::from([rom.to_dynamic()].as_slice()));
        let ctrl = Shared::from(Control::new(bank.clone()));
        // Construct self
        Self { ctrl, bank }
    }

    /// Gets a read-only reference to the boot ROM.
    #[must_use]
    pub fn rom(&self) -> ReadOnly<impl Device> {
        ReadOnly::from(self.bank.clone())
    }

    /// Gets a reference to the boot ROM's control register.
    #[must_use]
    pub fn ctrl(&self) -> Shared<Control> {
        self.ctrl.clone()
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
        // Control
        self.ctrl.reset();
        // Memory
        self.bank.reset();
    }
}

impl Board for Rom {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let boot = self.rom().to_dynamic();
        let ctrl = self.ctrl().to_dynamic();

        // Map devices on bus  // ┌──────┬────────┬──────────┬─────┐
                               // │ Addr │  Size  │   Name   │ Dev │
                               // ├──────┼────────┼──────────┼─────┤
        bus.map(0x0000, boot); // │ 0000 │  8 KiB │ Boot     │ ROM │
        bus.map(0xff50, ctrl); // │ ff50 │    1 B │ Control  │ Reg │
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
    bank: Shared<Bank>,
}

impl Control {
    /// Constructs a new `Control`.
    pub fn new(rom: Shared<Bank>) -> Self {
        Self { bank: rom }
    }
}

impl Address<u8> for Control {
    fn read(&self, _: usize) -> u8 {
        self.load()
    }

    fn write(&mut self, _: usize, value: u8) {
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

impl Device for Control {
    fn contains(&self, index: usize) -> bool {
        index < 1
    }

    fn len(&self) -> usize {
        1
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
