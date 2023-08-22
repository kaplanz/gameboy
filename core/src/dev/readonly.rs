use log::warn;
use remus::{Address, Block, Device};

/// Read-only device.
///
/// # Usage
///
/// `ReadOnly` provides a read-only view of the internal device, and ignoring
/// all writes which are logged as a warning.
#[derive(Debug)]
pub struct ReadOnly<D: Device>(D);

impl<D: Device> Address for ReadOnly<D> {
    fn read(&self, index: usize) -> u8 {
        self.0.read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        warn!("called `Device::write({index:#06x}, {value:#04x})` on a `ReadOnly`");
    }
}

impl<D: Device> Block for ReadOnly<D> {
    fn reset(&mut self) {
        self.0.reset();
    }
}

impl<D: Device> Device for ReadOnly<D> {
    fn contains(&self, index: usize) -> bool {
        self.0.contains(index)
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<D: Device> From<D> for ReadOnly<D> {
    fn from(dev: D) -> Self {
        Self(dev)
    }
}

#[cfg(test)]
mod tests {
    use remus::dev::Null;
    use remus::Dynamic;

    use super::*;

    fn setup() -> ReadOnly<Dynamic> {
        let rom = Null::<0x100>::with(0x55).to_dynamic();
        ReadOnly::from(rom)
    }

    #[test]
    fn from_works() {
        let ronly = setup();
        assert!((0x000..0x100)
            .map(|addr| ronly.read(addr))
            .all(|byte| byte == 0x55));
    }

    #[test]
    fn device_contains_works() {
        let ronly = setup();
        assert!((0x000..0x100).all(|addr| ronly.contains(addr)));
    }

    #[test]
    fn device_len_works() {
        let ronly = setup();
        assert_eq!(ronly.len(), 0x100);
    }

    #[test]
    fn device_read_works() {
        let ronly = setup();
        assert!((0x000..0x100)
            .map(|addr| ronly.read(addr))
            .all(|byte| byte == 0x55));
    }

    #[test]
    fn device_write_ignored() {
        let mut ronly = setup();
        (0x000..0x100).for_each(|addr| ronly.write(addr, 0xaa));
        assert!((0x000..0x100)
            .map(|addr| ronly.read(addr))
            .all(|byte| byte == 0x55));
    }
}
