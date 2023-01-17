//! Hardware blocks.
//!
//! Each of the following hardware models implements [`Block`](remus::Block).

#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

pub mod cart;

pub(crate) mod audio;
pub(crate) mod cpu;
pub(crate) mod joypad;
pub(crate) mod pic;
pub(crate) mod ppu;
pub(crate) mod serial;
pub(crate) mod timer;
