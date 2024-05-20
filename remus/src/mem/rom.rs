use super::{Error, Memory, Result};
use crate::{Byte, Word};

/// Read-only memory.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Rom<M: Memory>(M);

impl<M: Memory> Rom<M> {
    /// Gets a reference to the underlying value.
    pub fn inner(&self) -> &M {
        &self.0
    }

    /// Gets a mutable reference to the underlying value.
    pub fn inner_mut(&mut self) -> &mut M {
        &mut self.0
    }
}

impl<M: Memory> From<M> for Rom<M> {
    fn from(value: M) -> Self {
        Self(value)
    }
}

impl<M: Memory> Memory for Rom<M> {
    fn read(&self, addr: Word) -> Result<Byte> {
        self.0.read(addr)
    }

    fn write(&mut self, addr: Word, _: Byte) -> Result<()> {
        Err(if self.0.read(addr).is_ok() {
            Error::Misuse
        } else {
            Error::Range
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_read_works() {
        let ram = Rom::from([0; 0x100]);
        assert_eq!(ram.read(0u16).unwrap(), 0x00);
        assert!(matches!(ram.read(0x100u16), Err(Error::Range)));
    }

    #[test]
    fn memory_write_works() {
        let mut ram = Rom::from([0; 0x100]);
        assert!(matches!(ram.write(0, 0xaa), Err(Error::Misuse)));
        assert_eq!(ram.read(0).unwrap(), 0x00);
        assert!(matches!(ram.write(0x100, 0), Err(Error::Range)));
    }
}
