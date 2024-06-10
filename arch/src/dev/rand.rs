use crate::mem::{Memory, Result};
use crate::{Byte, Word};

/// Random device.
///
/// # Usage
///
/// The `Random` device ignores all writes, and always yields random "garbage"
/// values when read. This can be useful to allow memory accesses to an unmapped
/// region of memory without causing a panic.
#[derive(Debug, Default)]
pub struct Random;

impl Random {
    /// Constructs a new `Random`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Memory for Random {
    fn read(&self, _: Word) -> Result<Byte> {
        Ok(rand::random())
    }

    fn write(&mut self, _: Word, _: Byte) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let _ = Random::new();
    }

    #[test]
    fn memory_read_works() {
        let random = Random::new();
        (0..0x100).for_each(|addr| {
            let _ = random.read(addr).unwrap();
        });
    }

    #[test]
    fn memory_write_works() {
        let mut random = Random::new();
        (0..0x100).for_each(|addr| {
            let _ = random.write(addr, 0xaa);
        });
        (0..0x100).for_each(|addr| while random.read(addr) != Ok(0xaa) {});
    }
}
