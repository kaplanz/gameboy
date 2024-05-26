//! Emulator API.

use rugby_arch::Block;

pub mod audio;
pub mod cart;
pub mod joypad;
pub mod proc;
pub mod serial;
pub mod video;

/// Core interface.
pub trait Core:
    Block
    + audio::Support
    + cart::Support
    + joypad::Support
    + proc::Support
    + serial::Support
    + video::Support
{
}
