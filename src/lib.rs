//! # Game Boy
//!
//! Emulator implementations for the following Game Boy models:
//! - [`DMG`](crate::core::dmg): [Game Boy]
//!
//! # Examples
//!
//! ```
//! use rugby::core::dmg::{Cartridge, GameBoy};
//! use rugby::prelude::*;
//! use rugby_arch::Block; // for `Block::cycle`
//!
//! // Instantiate a cartridge from ROM bytes
//! let rom: &[u8]; // -- snip --
//! # rom = include_bytes!("../roms/games/2048/2048.gb");
//! let cart = Cartridge::new(rom).unwrap();
//!
//! // Create an emulator instance
//! let mut emu = GameBoy::new();
//! // Load the cartridge into the emulator
//! emu.load(cart);
//!
//! // Run the emulator
//! loop {
//!     emu.cycle();
//! #     break;
//! }
//! ```
//!
//! [Game Boy]: https://en.wikipedia.org/wiki/Game_Boy

#![warn(clippy::pedantic)]

pub mod app;
pub mod pal;

#[doc(inline)]
pub use rugby_core as core;

pub use crate::core::api as emu;

/// A prelude for conveniently writing emulator code.
///
/// The purpose of this module is to alleviate imports of many common traits by
/// adding a glob import to the top of modules:
///
/// ```rust
/// use rugby::prelude::*;
/// ```
///
/// Includes all core and library API traits.
#[rustfmt::skip]
pub mod prelude {
    // Application
    pub use crate::app::audio::Audio as _;
    pub use crate::app::joypad::Joypad as _;
    pub use crate::app::serial::Serial as _;
    pub use crate::app::video::Video as _;

    // Emulator
    pub use crate::emu::audio::{Audio as _, Support as _};
    pub use crate::emu::cart::{Cartridge as _, Support as _};
    pub use crate::emu::joypad::{Joypad as _, Support as _, };
    pub use crate::emu::proc::{Processor as _, Support as _};
    pub use crate::emu::serial::{Serial as _, Support as _};
    pub use crate::emu::video::{Video as _, Support as _};
}
