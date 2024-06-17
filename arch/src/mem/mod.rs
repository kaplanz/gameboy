//! Memory models.
//!
//! # Usage
//!
//! The [`Ram`] and [`Rom`] memory models work similarly to one another, with
//! the obvious exception that `Rom` returns an [error](Error::Misuse) on
//! writes.

use std::fmt::Debug;

use thiserror::Error;

use crate::reg::Register;
use crate::{Byte, Shared, Word};

mod ram;
mod rom;

pub use self::ram::Ram;
pub use self::rom::Rom;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Addressable memory-mapped interface.
pub trait Memory: Debug {
    /// Reads from the specified address.
    ///
    /// # Errors
    ///
    /// Errors if the device could not successfully be read from.
    fn read(&self, addr: Word) -> Result<Byte>;

    /// Writes to the specified address.
    ///
    /// # Errors
    ///
    /// Errors if the device could not successfully be written to.
    fn write(&mut self, addr: Word, data: Byte) -> Result<()>;
}

impl Memory for Byte {
    fn read(&self, _: Word) -> Result<Byte> {
        Ok(self.load())
    }

    fn write(&mut self, _: Word, data: Byte) -> Result<()> {
        self.store(data);
        Ok(())
    }
}

impl Memory for [Byte] {
    fn read(&self, addr: Word) -> Result<Byte> {
        self.get(usize::from(addr)).copied().ok_or(Error::Range)
    }

    fn write(&mut self, addr: Word, data: Byte) -> Result<()> {
        self.get_mut(usize::from(addr))
            .map(|val| *val = data)
            .ok_or(Error::Range)
    }
}

impl<const N: usize> Memory for [Byte; N] {
    fn read(&self, addr: Word) -> Result<Byte> {
        self.as_ref().read(addr)
    }

    fn write(&mut self, addr: Word, data: Byte) -> Result<()> {
        self.as_mut().write(addr, data)
    }
}

impl Memory for Box<[Byte]> {
    fn read(&self, addr: Word) -> Result<Byte> {
        self.as_ref().read(addr)
    }

    fn write(&mut self, addr: Word, data: Byte) -> Result<()> {
        self.as_mut().write(addr, data)
    }
}

impl<const N: usize> Memory for Box<[Byte; N]> {
    fn read(&self, addr: Word) -> Result<Byte> {
        self.as_ref().read(addr)
    }

    fn write(&mut self, addr: Word, data: Byte) -> Result<()> {
        self.as_mut().write(addr, data)
    }
}

impl<M: Memory + ?Sized> Memory for Shared<M> {
    fn read(&self, addr: Word) -> Result<Byte> {
        self.borrow().read(addr)
    }

    fn write(&mut self, addr: Word, data: Byte) -> Result<()> {
        self.borrow_mut().write(addr, data)
    }
}

impl Memory for Vec<Byte> {
    fn read(&self, addr: Word) -> Result<Byte> {
        self.as_slice().read(addr)
    }

    fn write(&mut self, addr: Word, data: Byte) -> Result<()> {
        self.as_mut_slice().write(addr, data)
    }
}

/// An error caused by a [memory](Memory) operation.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum Error {
    /// Device is unavailable.
    #[error("device is unavailable")]
    Busy,
    /// Unsupported operation.
    #[error("unsupported operation")]
    Misuse,
    /// Address out of range.
    #[error("address out of range")]
    Range,
}
