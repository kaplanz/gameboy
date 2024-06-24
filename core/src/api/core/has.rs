//! Component support.

use crate::api::part;

/// Audio support.
pub trait Audio {
    /// Audio interface.
    type Audio: part::audio::Audio;

    /// Borrows the core's audio.
    #[must_use]
    fn audio(&self) -> &Self::Audio;

    /// Mutably borrows the core's audio.
    #[must_use]
    fn audio_mut(&mut self) -> &mut Self::Audio;
}

/// Joypad support.
pub trait Joypad {
    /// Joypad interface.
    type Joypad: part::joypad::Joypad;

    /// Borrows the core's joypad.
    #[must_use]
    fn joypad(&self) -> &Self::Joypad;

    /// Mutably borrows the core's joypad.
    #[must_use]
    fn joypad_mut(&mut self) -> &mut Self::Joypad;
}

/// Processor support.
pub trait Processor {
    /// Compute interface.
    type Proc: part::proc::Processor;

    /// Borrows the core's processor.
    #[must_use]
    fn proc(&self) -> &Self::Proc;

    /// Mutably borrows the core's processor.
    #[must_use]
    fn proc_mut(&mut self) -> &mut Self::Proc;
}

/// Serial support.
pub trait Serial {
    /// Serial interface.
    type Serial: part::serial::Serial;

    /// Borrows the core's serial.
    #[must_use]
    fn serial(&self) -> &Self::Serial;

    /// Mutably borrows the core's serial.
    #[must_use]
    fn serial_mut(&mut self) -> &mut Self::Serial;
}

/// Video support.
pub trait Video {
    /// Video interface.
    type Video: part::video::Video;

    /// Borrows the core's video.
    #[must_use]
    fn video(&self) -> &Self::Video;

    /// Mutably borrows the core's video.
    #[must_use]
    fn video_mut(&mut self) -> &mut Self::Video;
}
