use log::warn;
use remus::dev::Null;
use remus::{Block, Device};

/// Unmapped device.
///
/// # Usage
///
/// The `Unmapped` device ialways yields the same "garbage" values when read,
/// and ignores all writes. This can be useful to warn of unmapped accesses
/// instead of causing a panic.
///
/// It behaves differently from [`Null`](remus::dev::Null) in that reads and
/// writes are logged, instead of completely ignored. Furthermore, it has a
/// default domain of the entire 16-bit address space.
#[derive(Debug)]
pub struct Unmapped<const N: usize = 0x10000>(Null<N>);

impl Unmapped {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for Unmapped {
    fn reset(&mut self) {
        self.0.reset();
    }
}

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
        warn!("called `Device::read({index:#06x})` on an `Unmapped`");
        self.0.read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        warn!("called `Device::write({index:#06x}, {value:#04x})` on an `Unmapped`");
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
