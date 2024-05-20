//! Hardware blocks.

#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

pub mod apu;
pub mod boot;
pub mod cart;
pub mod cpu;
pub mod dma;
pub mod joypad;
pub mod pic;
pub mod ppu;
pub mod serial;
pub mod timer;
