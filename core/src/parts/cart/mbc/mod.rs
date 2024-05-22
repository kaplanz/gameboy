//! Memory bank controllers.
//!
//! Implementations of various kinds of cartridge hardware.

#![allow(clippy::module_name_repetitions)]

use std::fmt::Debug;
use std::io;

use log::trace;
use remus::mio::{Bus, Device, Mmio};
use remus::{Block, Byte};

use super::header::Header;
use super::{Error, Info, Result};

mod bare;
mod mbc1;
mod mbc5;

pub use self::bare::Bare;
pub use self::mbc1::Mbc1;
pub use self::mbc5::Mbc5;

/// Memory data.
type Data = Box<[Byte]>;

/// Memory bank controller.
pub trait Mbc {
    /// Gets the contents of the cartridge's ROM.
    fn rom(&self) -> Device;

    /// Gets the contents of the cartridge's RAM.
    fn ram(&self) -> Device;

    /// Flashes the provided data onto the cartridge's RAM.
    ///
    /// # Errors
    ///
    /// May generate an I/O error indicating that the operation could not be
    /// completed. If an error is returned then it must be guaranteed that no
    /// bytes were read.
    fn flash(&mut self, buf: &mut impl io::Read) -> io::Result<usize>;

    /// Dumps the contents of the cartridge's RAM.
    ///
    /// # Errors
    ///
    /// May generate an I/O error indicating that the operation could not be
    /// completed. If an error is returned then no bytes were written.
    fn dump(&self, buf: &mut impl io::Write) -> io::Result<usize>;
}

/// Cartridge body.
///
/// Contains the cartridge's ROM and RAM, modelling cartridge-specific hardware.
#[derive(Debug)]
#[non_exhaustive]
pub enum Body {
    /// Bare ROM + RAM.
    Bare(Bare),
    /// MBC1 cartridge type.
    Mbc1(Mbc1),
    /// MBC5 cartridge type.
    Mbc5(Mbc5),
}

impl Body {
    /// Constructs a new `Body`.
    ///
    /// # Errors
    ///
    /// Returns an error if unsupported cartridge type is specified in the
    /// header.
    pub fn new(head: &Header, rom: &[Byte]) -> Result<Self> {
        // Initialize ROM
        let rom = make::rom(head, rom);
        if !rom.is_empty() {
            trace!("ROM:\n{rom}", rom = phex::Printer::<Byte>::new(0, &rom));
        }
        // Initialize RAM
        let ram = make::ram(head);
        if !ram.is_empty() {
            trace!("RAM:\n{ram}", ram = phex::Printer::<Byte>::new(0, &ram));
        }
        // Construct body
        match &head.info {
            &Info::Bare { .. } => Ok(Body::Bare(Bare::new(rom, ram))),
            &Info::Mbc1 { .. } => Ok(Body::Mbc1(Mbc1::new(rom, ram))),
            &Info::Mbc5 { .. } => Ok(Body::Mbc5(Mbc5::new(rom, ram))),
            kind => Err(Error::Unsupported(kind.clone())),
        }
    }
}

mod make {
    use std::cmp::Ordering;
    use std::iter;

    use log::{info, warn};
    use remus::Byte;

    use super::{Data, Header};

    /// Constructs a new ROM.
    pub fn rom(head: &Header, rom: &[Byte]) -> Data {
        let read = rom.len();
        match read.cmp(&head.romsz) {
            Ordering::Less => {
                warn!(
                    "loaded {init} bytes; remaining {diff} bytes uninitialized",
                    init = read,
                    diff = head.romsz - read
                );
            }
            Ordering::Equal => info!("loaded {read} bytes"),
            Ordering::Greater => {
                warn!(
                    "loaded {init} bytes; remaining {diff} bytes truncated",
                    init = head.romsz,
                    diff = read - head.romsz
                );
            }
        }
        rom.iter()
            .copied()
            // pad missing values with open bus value
            .chain(iter::repeat(0xff))
            // truncate based on recorded cartridge size
            .take(head.romsz)
            // collect as a heap-allocated slice
            .collect::<Box<_>>()
    }

    /// Constructs a new RAM.
    pub fn ram(head: &Header) -> Data {
        if head.info.has_ram() && head.ramsz == 0 {
            warn!("cartridge supports RAM, but specified size is zero");
        }
        if !head.info.has_ram() && head.ramsz > 0 {
            warn!(
                "cartridge does not support RAM, but specified size is non-zero (found: {})",
                head.ramsz
            );
        }
        vec![0; head.ramsz].into_boxed_slice()
    }
}

impl Block for Body {
    fn ready(&self) -> bool {
        match self {
            Body::Bare(mbc) => mbc.ready(),
            Body::Mbc1(mbc) => mbc.ready(),
            Body::Mbc5(mbc) => mbc.ready(),
        }
    }

    fn cycle(&mut self) {
        match self {
            Body::Bare(mbc) => mbc.cycle(),
            Body::Mbc1(mbc) => mbc.cycle(),
            Body::Mbc5(mbc) => mbc.cycle(),
        }
    }

    fn reset(&mut self) {
        match self {
            Body::Bare(mbc) => mbc.reset(),
            Body::Mbc1(mbc) => mbc.reset(),
            Body::Mbc5(mbc) => mbc.reset(),
        }
    }
}

impl Mbc for Body {
    fn rom(&self) -> Device {
        match self {
            Body::Bare(mbc) => mbc.rom(),
            Body::Mbc1(mbc) => mbc.rom(),
            Body::Mbc5(mbc) => mbc.rom(),
        }
    }

    fn ram(&self) -> Device {
        match self {
            Body::Bare(mbc) => mbc.ram(),
            Body::Mbc1(mbc) => mbc.ram(),
            Body::Mbc5(mbc) => mbc.ram(),
        }
    }

    /// Flashes the provided data onto the cartridge's RAM.
    fn flash(&mut self, buf: &mut impl io::Read) -> io::Result<usize> {
        match self {
            Body::Bare(mbc) => mbc.flash(buf),
            Body::Mbc1(mbc) => mbc.flash(buf),
            Body::Mbc5(mbc) => mbc.flash(buf),
        }
    }

    /// Dumps the contents of the cartridge's RAM.
    fn dump(&self, buf: &mut impl io::Write) -> io::Result<usize> {
        match self {
            Body::Bare(mbc) => mbc.dump(buf),
            Body::Mbc1(mbc) => mbc.dump(buf),
            Body::Mbc5(mbc) => mbc.dump(buf),
        }
    }
}

impl Mmio for Body {
    fn attach(&self, bus: &mut Bus) {
        bus.map(0x0000..=0x7fff, self.rom());
        bus.map(0xa000..=0xbfff, self.ram());
    }

    fn detach(&self, bus: &mut Bus) {
        assert!(bus.unmap(&self.rom()));
        assert!(bus.unmap(&self.ram()));
    }
}
