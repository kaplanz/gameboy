//! Memory bank controllers.
//!
//! Implementations of various kinds of cartridge hardware.

#![allow(clippy::module_name_repetitions)]

use std::cmp::Ordering;
use std::fmt::Debug;
use std::iter;

use log::{info, trace, warn};
use remus::mio::Mmio;
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
pub trait Mbc: Block + Debug + Mmio {
    /// Gets the cartridge's underlying ROM.
    fn rom(&self) -> Data;

    /// Gets the cartridge's underlying RAM.
    fn ram(&self) -> Data;
}

/// Constructs a memory bank controller from a parsed ROM and header.
pub(super) fn make(head: &Header, rom: &[Byte]) -> Result<Box<dyn Mbc>> {
    // Prepare ROM
    let rom = {
        // Calculate buffer stats
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
            .chain(iter::repeat(0xffu8))
            .take(head.romsz)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    };
    trace!("ROM:\n{rom}", rom = phex::Printer::<Byte>::new(0, &rom));

    // Declare RAM
    let ram = vec![0; head.ramsz].into_boxed_slice();

    // Construct MBC
    match &head.info {
        &Info::Bare { .. } => Ok(Box::new(Bare::new(rom, ram))),
        &Info::Mbc1 { .. } => Ok(Box::new(Mbc1::new(rom, ram))),
        &Info::Mbc5 { .. } => Ok(Box::new(Mbc5::new(rom, ram))),
        kind => Err(Error::Unsupported(kind.clone())),
    }
}
