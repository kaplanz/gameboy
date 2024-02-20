//! Game ROM cartridge.
//!
//! Encoded within the ROM is a [hardware specification][cartridge header] of
//! the physical cartridge on which the ROM is connected to the console.
//!
//! Additionally, one of several [memory bank controllers][mbcs] may be used to
//! expand the ROM and external RAM beyond the respective 32 KiB and 8 KiB
//! addressable bytes.
//!
//! [cartridge header]: https://gbdev.io/pandocs/The_Cartridge_Header.html
//! [mbcs]:             https://gbdev.io/pandocs/MBCs.html

use std::cmp::Ordering;
use std::iter;

use log::{debug, info, trace, warn};
use remus::bus::Mux;
use remus::dev::{Device, Dynamic, Null};
use remus::mem::{Ram, Rom};
use remus::{Block, Board};
use thiserror::Error;

use self::header::Kind;
use self::mbc::{Mbc, Mbc1, Mbc5, NoMbc};
use crate::arch::Bus;
use crate::dev::Unmapped;

mod header;

pub mod mbc;

pub use self::header::{Error as HeaderError, Header, LOGO};

/// Intermediate memory parts.
struct Memory {
    buf: Dynamic<u16, u8>,
    len: usize,
}

/// Cartridge model.
///
/// Parses a [`Header`] from the ROM, then initializes the memory bank
/// controller ([`mbc`]).
#[derive(Debug)]
pub struct Cartridge {
    // Metadata
    header: Header,
    // Memory
    mbc: Box<dyn Mbc>,
}

impl Cartridge {
    /// Constructs a new `Cartridge`.
    ///
    /// # Errors
    ///
    /// Returns an error when the cartridge header cannot be parsed.
    pub fn new(rom: &[u8]) -> Result<Self, Error> {
        // Parse cartridge header
        let header = Header::try_from(rom)?;
        debug!("header:\n{header}");

        // Construct memory bank controller
        let mbc = Self::mbc(&header, rom)?;

        Ok(Self { header, mbc })
    }

    /// Constructs a new `Cartridge` explicitly checking the entire header.
    ///
    /// # Errors
    ///
    /// Returns an error when the cartridge header contained an error.
    pub fn checked(rom: &[u8]) -> Result<Self, Error> {
        // Check then parse cartridge header
        let header = Header::check(rom).and_then(|()| Header::try_from(rom))?;
        debug!("header:\n{header}");

        // Construct memory bank controller
        let mbc = Self::mbc(&header, rom)?;

        Ok(Self { header, mbc })
    }

    /// Constructs a new `Cartridge` without checking the header.
    ///
    /// # Panics
    ///
    /// Panics if the memory bank controller could not be constructed.
    pub fn unchecked(rom: &[u8]) -> Self {
        // Parse cartridge header
        let header = Header::try_from(rom).ok().unwrap_or_else(Header::blank);
        debug!("header:\n{header}");

        // Construct memory bank controller
        let mbc = Self::mbc(&header, rom).ok().unwrap();

        Self { header, mbc }
    }

    /// Constructs a blank `Cartridge`.
    #[must_use]
    pub fn blank() -> Self {
        // Construct a blank header
        let header = Header::blank();

        // Use null devices for the ROM, RAM
        let rom = Null::<u8, 0x8000>::with(0xff).to_dynamic();
        let eram = Null::<u8, 0>::new().to_dynamic();

        // Construct a membory bank controller
        let mbc = Box::new(NoMbc::with(rom, eram));

        Self { header, mbc }
    }

    /// Gets a reference to the cartridge's header.
    #[must_use]
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Gets a shared reference to the cartridge's ROM.
    #[must_use]
    pub fn rom(&self) -> Dynamic<u16, u8> {
        self.mbc.rom()
    }

    /// Gets a shared reference to the cartridge's RAM.
    #[must_use]
    pub fn ram(&self) -> Dynamic<u16, u8> {
        self.mbc.ram()
    }

    /// Constructs a memory bank controller from a parsed ROM and header.
    #[allow(clippy::too_many_lines)]
    fn mbc(header: &Header, rom: &[u8]) -> Result<Box<dyn Mbc>, Error> {
        // Extract dimensions from header
        let &Header { romsz, ramsz, .. } = header;

        // Construct null device (for reuse where needed)
        let null = Unmapped::<0x2000>::new().to_dynamic();

        // Prepare ROM
        let rom = {
            // Calculate buffer stats
            let read = rom.len();
            match read.cmp(&header.romsz) {
                Ordering::Less => {
                    warn!(
                        "initialized {init} bytes; remaining {diff} bytes uninitialized",
                        init = read,
                        diff = header.romsz - read
                    );
                }
                Ordering::Equal => info!("initialized {read} bytes"),
                Ordering::Greater => {
                    warn!(
                        "initialized {init} bytes; remaining {diff} bytes truncated",
                        init = header.romsz,
                        diff = read - header.romsz
                    );
                }
            }
            rom.iter()
                .copied()
                .chain(iter::repeat(0xffu8))
                .take(romsz)
                .collect::<Vec<_>>()
                .into_boxed_slice()
        };
        trace!("ROM:\n{rom}", rom = phex::Printer::<u8>::new(0, &rom));

        // Construct ROM
        let rom = match romsz {
            0x0000_8000 => Rom::<u8, 0x0000_8000>::from(
                &*Box::<[_; 0x0000_8000]>::try_from(rom).map_err(Error::Mismatch)?,
            )
            .to_dynamic(),
            0x0001_0000 => Rom::<u8, 0x0001_0000>::from(
                &*Box::<[_; 0x0001_0000]>::try_from(rom).map_err(Error::Mismatch)?,
            )
            .to_dynamic(),
            0x0002_0000 => Rom::<u8, 0x0002_0000>::from(
                &*Box::<[_; 0x0002_0000]>::try_from(rom).map_err(Error::Mismatch)?,
            )
            .to_dynamic(),
            0x0004_0000 => Rom::<u8, 0x0004_0000>::from(
                &*Box::<[_; 0x0004_0000]>::try_from(rom).map_err(Error::Mismatch)?,
            )
            .to_dynamic(),
            0x0008_0000 => Rom::<u8, 0x0008_0000>::from(
                &*Box::<[_; 0x0008_0000]>::try_from(rom).map_err(Error::Mismatch)?,
            )
            .to_dynamic(),
            0x0010_0000 => Rom::<u8, 0x0010_0000>::from(
                &*Box::<[_; 0x0010_0000]>::try_from(rom).map_err(Error::Mismatch)?,
            )
            .to_dynamic(),
            0x0020_0000 => Rom::<u8, 0x0020_0000>::from(
                &*Box::<[_; 0x0020_0000]>::try_from(rom).map_err(Error::Mismatch)?,
            )
            .to_dynamic(),
            0x0040_0000 => Rom::<u8, 0x0040_0000>::from(
                &*Box::<[_; 0x0040_0000]>::try_from(rom).map_err(Error::Mismatch)?,
            )
            .to_dynamic(),
            0x0080_0000 => Rom::<u8, 0x0080_0000>::from(
                &*Box::<[_; 0x0080_0000]>::try_from(rom).map_err(Error::Mismatch)?,
            )
            .to_dynamic(),
            _ => unreachable!(),
        };

        // Construct RAM
        let ram = match ramsz {
            0x00000 => null.clone().to_dynamic(),
            0x02000 => Ram::<u8, 0x02000>::new().to_dynamic(),
            0x08000 => Ram::<u8, 0x08000>::new().to_dynamic(),
            0x20000 => Ram::<u8, 0x20000>::new().to_dynamic(),
            0x10000 => Ram::<u8, 0x10000>::new().to_dynamic(),
            _ => unreachable!(),
        };

        // Construct a memory bank controller
        let mbc: Box<dyn Mbc> = match &header.cart {
            &Kind::NoMbc { ram: has_ram, .. } => {
                let ram = [null, ram][has_ram as usize].clone();
                Box::new(NoMbc::with(rom, ram))
            }
            &Kind::Mbc1 { ram: has_ram, .. } => {
                let rom = Memory {
                    buf: rom,
                    len: romsz,
                };
                let ram = Memory {
                    buf: [null, ram][has_ram as usize].clone(),
                    len: ramsz.max(0x2000),
                };
                Box::new(Mbc1::with(rom, ram))
            }
            &Kind::Mbc5 { ram: has_ram, .. } => {
                let rom = Memory {
                    buf: rom,
                    len: romsz,
                };
                let ram = Memory {
                    buf: [null, ram][has_ram as usize].clone(),
                    len: ramsz,
                };
                Box::new(Mbc5::with(rom, ram))
            }
            cart => return Err(Error::Unimplemented(cart.clone())),
        };

        Ok(mbc)
    }
}

impl Block for Cartridge {
    fn reset(&mut self) {
        // Memory
        self.mbc.reset();
    }
}

impl Board<u16, u8> for Cartridge {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let rom = self.rom();
        let ram = self.ram();

        // Map devices on bus          // ┌──────┬────────┬────────────┬─────┐
                                       // │ Addr │  Size  │    Name    │ Dev │
                                       // ├──────┼────────┼────────────┼─────┤
        bus.map(0x0000..=0x7fff, rom); // │ 0000 │ 32 KiB │ Cartridge  │ ROM │
        bus.map(0xa000..=0xbfff, ram); // │ a000 │  8 KiB │ External   │ RAM │
                                       // └──────┴────────┴────────────┴─────┘
    }

    fn disconnect(&self, bus: &mut Bus) {
        // Extract devices
        let rom = self.rom();
        let ram = self.ram();

        // Unmap devices on bus
        assert!(bus.unmap(&rom).is_some());
        assert!(bus.unmap(&ram).is_some());
    }
}

/// A type specifying categories of [`Cartridge`] error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("could not parse header")]
    Header(#[from] header::Error),
    #[error("cartridge size mismatch")]
    Mismatch(Box<[u8]>),
    #[error("unimplemented cartridge kind: {0}")]
    Unimplemented(Kind),
}
