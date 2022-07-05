//! # Game Boy Core
//!
//! This library implements the core behaviour of the various hardware
//! components of the Nintendo Game Boy family of consoles.

mod dev;
mod emu;
mod hw;
mod model;

pub use self::emu::Emulator;
pub use self::model::dmg;
