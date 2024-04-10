//! # Game Boy Core.
//!
//! This library implements the core behaviour of the various hardware
//! components of the Nintendo Game Boy family of consoles.

#![warn(clippy::pedantic)]
#![allow(clippy::similar_names)]

pub(crate) mod dev;
pub(crate) mod hw;
pub(crate) mod model;

pub mod api;

pub use crate::model::dmg;
