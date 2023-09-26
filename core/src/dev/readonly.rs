use log::warn;
use remus::dev::Device;
use remus::{Address, Block};

/// Read-only device.
///
/// # Usage
///
/// `ReadOnly` provides a read-only view of the internal device, and ignoring
/// all writes which are logged as a warning.
#[derive(Debug)]
pub struct ReadOnly<T: Device<u16, u8>>(T);

impl<T> Address<u16, u8> for ReadOnly<T>
where
    T: Device<u16, u8>,
{
    fn read(&self, index: u16) -> u8 {
        self.0.read(index)
    }

    fn write(&mut self, index: u16, value: u8) {
        warn!("called `Device::write({index:#06x}, {value:#04x})` on a `ReadOnly`");
    }
}

impl<T: Device<u16, u8>> Block for ReadOnly<T> {
    fn reset(&mut self) {
        self.0.reset();
    }
}

impl<T: Device<u16, u8>> Device<u16, u8> for ReadOnly<T> {}

impl<T: Device<u16, u8>> From<T> for ReadOnly<T> {
    fn from(dev: T) -> Self {
        Self(dev)
    }
}

#[cfg(test)]
mod tests {
    use remus::dev::{Dynamic, Null};

    use super::*;

    fn setup() -> ReadOnly<Dynamic<u16, u8>> {
        let rom = Null::<u8, 0x100>::with(0x55).to_dynamic();
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
