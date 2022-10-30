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

use log::{debug, error, info, trace};
use remus::dev::Null;
use remus::mem::{Ram, Rom};
use remus::{Block, Device, Memory, SharedDevice};
use thiserror::Error;

use self::header::CartridgeType;
use self::mbc::{Mbc, Mbc1, NoMbc};

mod header;

pub mod mbc;

pub use self::header::{Error as HeaderError, Header};

/// Cartridge model.
///
/// Parses a [`Header`] from the ROM, then initializes the memory bank
/// controller ([`mbc`]).
#[derive(Debug)]
pub struct Cartridge {
    header: Header,
    mbc: Box<dyn Mbc>,
}

impl Cartridge {
    /// Constructs a new `Cartridge`.
    pub fn new(rom: &[u8]) -> Result<Self, Error> {
        // Parse cartridge header
        let header = Header::try_from(rom)?;
        debug!("Header:\n{header}");

        // Construct null device (for reuse where needed)
        let null = Null::<0>::new().to_shared();

        // Prepare external ROM
        let rom = {
            // Calculate buffer stats
            let read = rom.len();
            match read.cmp(&header.romsz) {
                Ordering::Less => {
                    error!(
                        "Initialized {read} bytes; remaining {diff} bytes uninitialized",
                        diff = header.romsz - read
                    )
                }
                Ordering::Equal => info!("Initialized {read} bytes"),
                Ordering::Greater => {
                    error!(
                        "Initialized {read} bytes; remaining {diff} bytes truncated",
                        diff = read - header.romsz
                    )
                }
            }
            rom.iter()
                .cloned()
                .chain(iter::repeat(0u8))
                .take(header.romsz)
                .collect::<Vec<_>>()
                .into_boxed_slice()
        };
        trace!("ROM:\n{}", &&*rom as &dyn Memory);

        // Construct external ROM
        let rom = {
            match header.romsz {
                0x8000 => {
                    Rom::<0x8000>::from(&*Box::<[_; 0x8000]>::try_from(rom).unwrap()).to_shared()
                }
                0x10000 => {
                    Rom::<0x10000>::from(&*Box::<[_; 0x10000]>::try_from(rom).unwrap()).to_shared()
                }
                0x20000 => {
                    Rom::<0x20000>::from(&*Box::<[_; 0x20000]>::try_from(rom).unwrap()).to_shared()
                }
                0x40000 => {
                    Rom::<0x40000>::from(&*Box::<[_; 0x40000]>::try_from(rom).unwrap()).to_shared()
                }
                0x80000 => {
                    Rom::<0x80000>::from(&*Box::<[_; 0x80000]>::try_from(rom).unwrap()).to_shared()
                }
                0x100000 => Rom::<0x100000>::from(&*Box::<[_; 0x100000]>::try_from(rom).unwrap())
                    .to_shared(),
                0x200000 => Rom::<0x200000>::from(&*Box::<[_; 0x200000]>::try_from(rom).unwrap())
                    .to_shared(),
                0x400000 => Rom::<0x400000>::from(&*Box::<[_; 0x400000]>::try_from(rom).unwrap())
                    .to_shared(),
                0x800000 => Rom::<0x800000>::from(&*Box::<[_; 0x800000]>::try_from(rom).unwrap())
                    .to_shared(),
                _ => unreachable!(),
            }
        };

        // Construct external RAM
        let eram = match header.ramsz {
            0x0 => null.clone(),
            0x2000 => Ram::<0x2000>::new().to_shared(),
            0x8000 => Ram::<0x8000>::new().to_shared(),
            0x20000 => Ram::<0x20000>::new().to_shared(),
            0x10000 => Ram::<0x10000>::new().to_shared(),
            _ => unreachable!(),
        };

        // Construct a cartridge
        let mbc: Box<dyn Mbc> = match header.cart {
            CartridgeType::NoMbc { ram, .. } => {
                let eram = [null, eram][ram as usize].clone();
                Box::new(NoMbc::with(rom, eram))
            }
            CartridgeType::Mbc1 { ram, battery } => {
                let eram = [null, eram][ram as usize].clone();
                Box::new(Mbc1::with(rom, eram, battery))
            }
            cart => unimplemented!("{cart:?}"),
        };

        Ok(Self { header, mbc })
    }

    /// Gets a reference to the cartridge's header.
    #[must_use]
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Gets a shared reference to the cartridge's ROM.
    pub fn rom(&self) -> SharedDevice {
        self.mbc.rom()
    }

    /// Gets a shared reference to the cartridge's RAM.
    pub fn ram(&self) -> SharedDevice {
        self.mbc.ram()
    }
}

impl Block for Cartridge {
    fn reset(&mut self) {
        // Reset MBC
        self.mbc.reset();
    }
}

impl Default for Cartridge {
    fn default() -> Self {
        let rom = [
            0xc3, 0x8b, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc3, 0x8b, 0x02, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x87, 0xe1,
            0x5f, 0x16, 0x00, 0x19, 0x5e, 0x23, 0x56, 0xd5, 0xe1, 0xe9, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xc3, 0xfd, 0x01, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xc3, 0x12, 0x27, 0xff, 0xff, 0xff, 0xff, 0xff, 0xc3, 0x12, 0x27, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xc3, 0x7e, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0x00, 0xc3, 0x50, 0x01, 0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d,
            0x00, 0x0b, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0c, 0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f,
            0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb,
            0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0xe6, 0x00, 0x6b,
        ];
        Self {
            header: Header::try_from(&rom[..]).unwrap(),
            mbc: Box::new(NoMbc::with(
                Rom::<0x8000>::new().to_shared(),
                Ram::<0x2000>::new().to_shared(),
            )),
        }
    }
}

/// A type specifying general categories of [`Cartridge`] error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("could not parse header")]
    Header(#[from] header::Error),
}
