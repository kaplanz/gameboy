//! Hardware blocks.
//!
//! Each of the following hardware models implements [`Block`](remus::Block).

pub mod cart;

pub(crate) mod cpu;
pub(crate) mod joypad;
pub(crate) mod pic;
pub(crate) mod ppu;
pub(crate) mod timer;
