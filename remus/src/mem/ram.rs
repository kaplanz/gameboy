use super::{Memory, Result};
use crate::{Byte, Word};

/// Random-access memory.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Ram<M: Memory>(M);

impl<M: Memory> Ram<M> {
    /// Gets a reference to the underlying value.
    pub fn inner(&self) -> &M {
        &self.0
    }

    /// Gets a mutable reference to the underlying value.
    pub fn inner_mut(&mut self) -> &mut M {
        &mut self.0
    }
}

impl<M: Memory> From<M> for Ram<M> {
    fn from(value: M) -> Self {
        Self(value)
    }
}

impl<M: Memory> Memory for Ram<M> {
    fn read(&self, addr: Word) -> Result<Byte> {
        self.0.read(addr)
    }

    fn write(&mut self, addr: Word, data: Byte) -> Result<()> {
        self.0.write(addr, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::Error;

    #[test]
    fn memory_read_works() {
        let ram = Ram::from([0; 0x100]);
        assert_eq!(ram.read(0).unwrap(), 0x00);
        assert!(matches!(ram.read(0x100), Err(Error::Range)));
    }

    #[test]
    fn memory_write_works() {
        let mut ram = Ram::from([0; 0x100]);
        ram.write(0u16, 0xaa).unwrap();
        assert_eq!(ram.read(0).unwrap(), 0xaa);
        assert!(matches!(ram.write(0x100, 0), Err(Error::Range)));
    }
}
