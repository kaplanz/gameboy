//! Memory bank controllers.
//!
//! Implementations of cartridge memory bank controllers.

#![allow(clippy::module_name_repetitions)]

use std::fmt::Debug;

use remus::bus::adapt::View;
use remus::dev::{Device, Dynamic};
use remus::Block;

use super::Memory;

mod mbc1;
mod mbc5;
mod nombc;

pub use self::mbc1::Mbc1;
pub use self::mbc5::Mbc5;
pub use self::nombc::NoMbc;

/// Unified MBC interface.
pub(super) trait Mbc: Block + Debug {
    /// Gets a shared reference to the MBC's ROM.
    fn rom(&self) -> Dynamic<u16, u8>;

    /// Gets a shared reference to the MBC's RAM.
    fn ram(&self) -> Dynamic<u16, u8>;
}

impl Memory {
    /// Fractures the memory into a bank.
    fn fracture(self, chunk: usize) -> Vec<Dynamic<u16, u8>> {
        // Determine how many banks are needed
        let nbanks = self.len / chunk;
        // Create chunked views of the memory
        (0..nbanks)
            .map(|i| {
                let head = u16::try_from(chunk * i).unwrap();
                let tail = u16::try_from(chunk * (i + 1) - 1).unwrap();
                View::new(head..=tail, self.buf.clone()).to_dynamic()
            })
            .collect()
    }
}
