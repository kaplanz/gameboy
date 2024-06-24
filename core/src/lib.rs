//! # Game Boy Core.
//!
//! This library implements the core behaviour of the various hardware
//! components of the Nintendo Game Boy family of consoles.

#![warn(clippy::pedantic)]
#![allow(clippy::similar_names)]

mod model;
mod parts;

/// Emulator API.
pub mod api {
    pub mod core;
    pub mod part;
}

pub use crate::model::dmg;
