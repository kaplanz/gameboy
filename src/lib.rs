//! # Game Boy
//!
//! Emulator implementations for the following Game Boy models:
//! - [`DMG`](crate::core::dmg): [Game Boy]
//!
//! # Examples
//!
//! ```
//! use rugby::emu::cart::Support;
//! use rugby::core::dmg::cart::Cartridge;
//! use rugby::core::dmg::GameBoy;
//! use remus::Block; // for `Block::cycle`
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
