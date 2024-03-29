//! # Game Boy
//!
//! Emulator implementations for the following Game Boy models:
//! - [`DMG`](crate::core::dmg): [Game Boy]
//!
//! # Examples
//!
//! ```
//! use gameboy::core::dmg::cart::Cartridge;
//! use gameboy::core::dmg::GameBoy;
//! use remus::Machine; // for `Machine::cycle`
//!
//! // Instantiate a cartridge from ROM bytes
//! let rom: &[u8]; // -- snip --
//! # rom = include_bytes!("../roms/games/gbmines.gb");
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

mod api;

#[cfg(feature = "gbd")]
pub mod gbd;

pub use api::*;
#[doc(inline)]
pub use gameboy_core as core;
