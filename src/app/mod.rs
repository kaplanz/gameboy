//! Frontend API.

use crate::emu::Core;

pub mod audio;
pub mod joypad;
pub mod serial;
pub mod video;

use self::audio::Audio;
use self::joypad::Joypad;
use self::serial::Serial;
use self::video::Video;

/// Frontend interface.
pub trait Frontend: Audio + Joypad + Serial + Video {
    /// Emulator core.
    type Core: Core;
}
