use log::warn;
use remus::dev::Null;
use remus::{Block, Device};

/// Unmapped device.
///
/// # Usage
///
/// The [`Unmapped`] device ignores all writes, and always yields the same
/// "garbage" values when read. This can be useful to allow memory accesses to
/// an unmapped region of memory without causing a panic.
///
/// Additionally, it behaves differently from [`Null`](remus::dev::Null) in that
/// its scope is the entirety of the 16-bit address space. Furthermore, any
/// reads or writes are logged as a warning.
#[derive(Debug)]
pub struct Unmapped(Null<0x10000>);

impl Unmapped {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for Unmapped {}

impl Default for Unmapped {
    fn default() -> Self {
        Self(Null::<0x10000>::with(0xff))
    }
}

impl Device for Unmapped {
    fn contains(&self, index: usize) -> bool {
        (0..self.len()).contains(&index)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn read(&self, index: usize) -> u8 {
        warn!("called `Device::read({index:#06x})` on a `Unmapped`");
        self.0.read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        warn!("called `Device::write({index:#06x}, {value:#04x})` on a `Unmapped`");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let unmapped = Unmapped::new();
        assert!((0x000..0x100)
            .map(|addr| unmapped.read(addr))
            .all(|byte| byte == 0xff));
    }

    #[test]
    fn device_contains_works() {
        let unmapped = Unmapped::new();
        assert!((0x000..0x100).all(|addr| unmapped.contains(addr)));
    }

    #[test]
    fn device_len_works() {
        assert_eq!(Unmapped::new().len(), 0x10000)
    }

    #[test]
    fn device_read_works() {
        let unmapped = Unmapped::new();
        assert!((0x000..0x100)
            .map(|addr| unmapped.read(addr))
            .all(|byte| byte == 0xff));
    }

    #[test]
    fn device_write_works() {
        let mut unmapped = Unmapped::new();
        (0x000..0x100).for_each(|addr| unmapped.write(addr, 0xaa));
        assert!((0x000..0x100)
            .map(|addr| unmapped.read(addr))
            .all(|byte| byte == 0xff));
    }
}
