use crate::mem::{Memory, Result};
use crate::{Byte, Word};

/// Null device.
///
/// # Usage
///
/// The `Null` device ignores all writes, and always yields the same "garbage"
/// values when read. This can be useful to allow memory accesses to an unmapped
/// region of memory without causing a panic.
///
/// `Null` defaults to yielding the null byte (`0x00`) when read, but this can
/// be changed either by constructing with [`Null::with`], or through the
/// [`Null::value`] method at runtime.
#[derive(Debug, Default)]
pub struct Null(Byte);

impl Null {
    /// Constructs a new `Null` device.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct an instance of [`Null`] that yields the specified value when
    /// performing a read.
    #[must_use]
    pub fn with(value: Byte) -> Self {
        Self(value)
    }

    /// Set the value to be used when performing a read.
    pub fn value(&mut self, value: Byte) {
        self.0 = value;
    }
}

impl Memory for Null {
    fn read(&self, _: Word) -> Result<Byte> {
        Ok(self.0)
    }

    fn write(&mut self, _: Word, _: Byte) -> Result<()> {
        Ok(())
    }
}

#[allow(clippy::items_after_statements)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let null = Null::new();
        assert!((0..0x100)
            .map(|addr| null.read(addr))
            .all(|data| data == Ok(0)));
    }

    #[test]
    fn with_works() {
        let null = Null::with(0xaa);
        assert!((0..0x100)
            .map(|addr| null.read(addr))
            .all(|data| data == Ok(0xaa)));
    }

    #[test]
    fn memory_read_works() {
        let null = Null::with(0xaa);
        assert!((0..0x100)
            .map(|addr| null.read(addr))
            .all(|data| data == Ok(null.0)));
    }

    #[test]
    fn memory_write_works() {
        let mut null = Null::new();
        (0..0x100).for_each(|addr| null.write(addr, 0xaa).unwrap());
        assert!((0..0x100)
            .map(|addr| null.read(addr))
            .all(|data| data == Ok(0)));
    }
}
