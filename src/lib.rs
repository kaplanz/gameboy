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
//! // Instantiate a `Cartridge`
//! let rom: &[u8]; // -- snip --
//! # rom = include_bytes!("../roms/games/2048/2048.gb");
//! let cart = Cartridge::new(rom).unwrap();
//!
//! // Create a `GameBoy` instance
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

pub use gameboy_core as core;

pub mod api;
