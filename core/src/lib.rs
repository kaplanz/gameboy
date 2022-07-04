//! # Game Boy Core
//!
//! This library implements the core behaviour of the various hardware
//! components of the Nintendo Game Boy family of consoles.

pub use crate::emu::Emulator;
pub use crate::model::dmg::GameBoy;

pub mod dev;
mod emu;
pub mod hw;
pub mod io;
pub mod model;
pub mod spec;
pub mod util;
