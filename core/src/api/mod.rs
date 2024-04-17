//! Emulator API.

use remus::Machine;

pub mod audio;
pub mod cart;
pub mod joypad;
pub mod proc;
pub mod serial;
pub mod video;

/// Core interface.
pub trait Core:
    Machine
    + audio::Support
    + cart::Support
    + joypad::Support
    + proc::Support
    + serial::Support
    + video::Support
{
}