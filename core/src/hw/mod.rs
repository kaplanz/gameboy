//! Hardware blocks.
//!
//! Each of the following hardware models implements [`Block`](remus::Block).

#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

pub mod cart;
pub mod cpu;
pub mod ppu;
pub mod timer;

pub(crate) mod apu;
pub(crate) mod joypad;
pub(crate) mod pic;
pub(crate) mod serial;
