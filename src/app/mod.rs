//! Frontend API.

use crate::emu::core::Core;

pub mod audio;
pub mod joypad;
pub mod serial;
pub mod video;

use self::audio::Audio;
use self::joypad::Joypad;
use self::serial::Serial;
use self::video::Video;

/// Emulator frontend.
pub trait Frontend: Audio + Joypad + Serial + Video {
    /// Emulator core.
    type Core: Core;
}
