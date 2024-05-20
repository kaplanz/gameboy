//! Memory interface.
//!
//! # Usage
//!
//! The [`Bus`] struct allows the user to mount another [`Device`] to
//! anywhere within the address space. As it itself implements `Device`, it
//! may be mapped in a nested fashion.
//!
//! Together with the [adapters](self::adapt), `Bus` is the primary method of
//! emulating [memory-mapped I/O].
//!
//! [memory-mapped I/O]: https://en.wikipedia.org/wiki/Memory-mapped_I/O

use std::fmt::Debug;
use std::ops::RangeInclusive;

use self::imp::Map;
use super::Device;
use crate::mem::{Error, Memory, Result};
use crate::{Byte, Word};

mod imp;

/// Mappable address range.
type Range = RangeInclusive<Word>;

/// Address [bus].
///
/// [bus]: https://en.wikipedia.org/wiki/Bus_(computing)
#[derive(Debug, Default)]
pub struct Bus {
    map: Map,
}

impl Bus {
    /// Constructs a new, empty `Bus`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears the bus, removing all devices.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Maps a device to the provided range.
    pub fn map(&mut self, range: Range, dev: Device) {
        self.map.map(range, dev);
    }

    /// Unmaps and returns a device.
    ///
    /// Returns `None` if device is not mapped.
    pub fn unmap(&mut self, dev: &Device) -> bool {
        self.map.unmap(dev)
    }
}

impl<const N: usize> From<[(Range, Device); N]> for Bus {
    fn from(arr: [(Range, Device); N]) -> Self {
        let mut this = Self::default();
        for (range, dev) in arr {
            this.map(range, dev);
        }
        this
    }
}

impl Memory for Bus {
    fn read(&self, addr: Word) -> Result<Byte> {
        self.map
            .select(addr)
            .flat_map(|it| it.entry.borrow().read(addr - it.base()))
            .next()
            .ok_or(Error::Range)
    }

    fn write(&mut self, addr: Word, data: Byte) -> Result<()> {
        self.map
            .select(addr)
            .flat_map(|it| it.entry.borrow_mut().write(addr - it.base(), data))
            .next()
            .ok_or(Error::Range)
    }
}

#[allow(clippy::items_after_statements)]
#[allow(clippy::range_plus_one)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::Ram;

    fn setup() -> Bus {
        Bus::from([
            (0x000..=0x0ff, Device::dev(Ram::from([0; 0x100]))),
            (0x100..=0x1ff, Device::dev(Ram::from([1; 0x100]))),
            (0x200..=0x2ff, Device::dev(Ram::from([2; 0x100]))),
        ])
    }

    #[test]
    fn new_works() {
        let bus = Bus::new();
        assert_eq!(bus.map.iter().count(), 0);
    }

    #[test]
    fn from_works() {
        setup();
    }

    #[test]
    fn clear_works() {
        let mut bus = setup();
        bus.clear();
        assert_eq!(bus.map.iter().count(), 0);
    }

    #[test]
    fn map_works() {
        let bus = setup();
        assert!((0x000..=0x0ff)
            .map(|idx| bus.read(idx))
            .all(|value| value == Ok(0)));
        assert!((0x100..=0x1ff)
            .map(|idx| bus.read(idx))
            .all(|value| value == Ok(1)));
        assert!((0x200..=0x2ff)
            .map(|idx| bus.read(idx))
            .all(|value| value == Ok(2)));
    }

    #[test]
    fn map_overlapping_works() {
        let a = Bus::from([(0x0000..=0x9fff, Device::dev(Ram::from([0x55; 0xa000])))]);
        let b = Bus::from([(0x6000..=0xffff, Device::dev(Ram::from([0xaa; 0xa000])))]);
        let c = Bus::from([
            (0x0000..=0xffff, Device::dev(a)),
            (0x0000..=0xffff, Device::dev(b)),
        ]);
        assert_eq!(
            [
                c.read(0x0000),
                c.read(0x5fff),
                c.read(0x6000),
                c.read(0x9fff),
                c.read(0xa000),
                c.read(0xffff),
            ],
            [Ok(0x55), Ok(0x55), Ok(0x55), Ok(0x55), Ok(0xaa), Ok(0xaa)]
        );
    }

    #[test]
    fn unmap_works() {
        let mut bus = Bus::new();
        let ram = Ram::from([0; 0x100]);
        let dev = Device::dev(ram);
        bus.map(0x000..=0x0ff, dev.clone());
        assert!(bus.unmap(&dev));
        assert!(bus.read(0).is_err());
    }

    #[test]
    fn memory_read_mapped_works() {
        let bus = setup();
        (0x000..0x100).for_each(|i| assert_eq!(bus.read(i), Ok(0)));
        (0x100..0x200).for_each(|i| assert_eq!(bus.read(i), Ok(1)));
        (0x200..0x300).for_each(|i| assert_eq!(bus.read(i), Ok(2)));
    }

    #[test]
    #[should_panic = "address is unmapped"]
    fn memory_read_unmapped_panics() {
        let bus = setup();
        bus.read(0x301).expect("address is unmapped");
    }

    #[test]
    fn memory_write_mapped_works() {
        let mut bus = setup();
        (0x000..0x300).for_each(|i| bus.write(i, 4).unwrap());
        (0x000..0x300).for_each(|i| assert_eq!(bus.read(i), Ok(4)));
    }

    #[test]
    #[should_panic = "address is unmapped"]
    fn memory_write_unmapped_panics() {
        let mut bus = setup();
        bus.write(0x301, 4).expect("address is unmapped");
    }

    #[allow(clippy::range_minus_one)]
    #[allow(clippy::reversed_empty_ranges)]
    #[test]
    fn memory_read_write_overlapping_mapped_works() {
        // Let's create a relatively complicated overlapping bus:
        //     ┌─────────────────────────────────────────────────┐
        // D0: │                 a                               │
        // D1: │                  bb                             │
        // D2: │                    cccc                         │
        // D3: │                        ddddddddd                │
        // D4: │ eeeeeeeeeeeeeeee                                │
        // D5: │ ffffffffffffffffffffffffffffffffffffffffffff... │
        //     ├─────────────────────────────────────────────────┤
        //     │ eeeeeeeeeeeeeeeeabbccccdddddddddffffffffffff... │
        //     └─────────────────────────────────────────────────┘
        let mut bus = Bus::new();
        // Device 0
        const N0: u16 = 1;
        const A0: u16 = 0x1000;
        let d0 = Device::dev(Ram::from([0xaa; N0 as usize]));
        bus.map(A0..=A0 + N0 - 1, d0);
        // Device 1
        const N1: u16 = 2;
        const A1: u16 = A0 + N0;
        let d1 = Device::dev(Ram::from([0xbb; N1 as usize]));
        bus.map(A1..=A1 + N1 - 1, d1);
        // Device 2
        const N2: u16 = 4;
        const A2: u16 = A1 + N1;
        let d2 = Device::dev(Ram::from([0xcc; N2 as usize]));
        bus.map(A2..=A2 + N2 - 1, d2);
        // Device 3
        const N3: u16 = 8;
        const A3: u16 = A2 + N2;
        let d3 = Device::dev(Ram::from([0xdd; N3 as usize]));
        bus.map(A3..=A3 + N3 - 1, d3);
        // Device 4
        const N4: u16 = 16;
        const A4: u16 = 0;
        let d4 = Device::dev(Ram::from([0xee; N4 as usize]));
        bus.map(A4..=A4 + N4 - 1, d4);
        // Device 5
        const N5: u16 = 128;
        const A5: u16 = A4;
        let d5 = Device::dev(Ram::from([0xff; N5 as usize]));
        bus.map(A5..=A5 + N5 - 1, d5);

        // Check if it is accessed properly...
        assert!((A0..A0 + N0)
            .map(|index| bus.read(index))
            .all(|byte| byte == Ok(0xaa)));
        assert!((A1..A1 + N1)
            .map(|index| bus.read(index))
            .all(|byte| byte == Ok(0xbb)));
        assert!((A2..A2 + N2)
            .map(|index| bus.read(index))
            .all(|byte| byte == Ok(0xcc)));
        assert!((A3..A3 + N3)
            .map(|index| bus.read(index))
            .all(|byte| byte == Ok(0xdd)));
        assert!((A4..A4 + N4)
            .map(|index| bus.read(index))
            .all(|byte| byte == Ok(0xee)));
        assert!((A3 + N3..A5 + N5)
            .map(|index| bus.read(index))
            .all(|byte| byte == Ok(0xff)));
    }
}
