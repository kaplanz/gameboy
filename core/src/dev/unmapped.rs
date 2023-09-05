use log::warn;
use remus::dev::{Device, Null};
use remus::{Address, Block};

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

impl<const N: usize> Unmapped<N> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<const N: usize> Address<u8> for Unmapped<N> {
    fn read(&self, index: usize) -> u8 {
        warn!("called `Device::read({index:#06x})` on an `Unmapped`");
        self.0.read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        warn!("called `Device::write({index:#06x}, {value:#04x})` on an `Unmapped`");
    }
}

impl<const N: usize> Block for Unmapped<N> {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl<const N: usize> Default for Unmapped<N> {
    fn default() -> Self {
        Self(Null::with(0xff))
    }
}

impl<const N: usize> Device for Unmapped<N> {
    fn contains(&self, index: usize) -> bool {
        (0..self.len()).contains(&index)
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let unmap = Unmapped::<0x10000>::new();
        assert!((0x000..0x100)
            .map(|addr| unmap.read(addr))
            .all(|byte| byte == 0xff));
    }

    #[test]
    fn device_contains_works() {
        let unmap = Unmapped::<0x10000>::new();
        assert!((0x000..0x100).all(|addr| unmap.contains(addr)));
    }

    #[test]
    fn device_len_works() {
        assert_eq!(Unmapped::<0x10000>::new().len(), 0x10000);
    }

    #[test]
    fn device_read_works() {
        let unmap = Unmapped::<0x10000>::new();
        assert!((0x000..0x100)
            .map(|addr| unmap.read(addr))
            .all(|byte| byte == 0xff));
    }

    #[test]
    fn device_write_works() {
        let mut unmap = Unmapped::<0x10000>::new();
        (0x000..0x100).for_each(|addr| unmap.write(addr, 0xaa));
        assert!((0x000..0x100)
            .map(|addr| unmap.read(addr))
            .all(|byte| byte == 0xff));
    }
}
