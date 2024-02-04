//! Frontend API.

pub use self::audio::*;
pub use self::joypad::*;
pub use self::serial::*;
pub use self::video::*;

/// Frontend interface.
pub trait Frontend {
    /// A type specifying [`Frontend`] errors.
    type Error;

    /// Audio interface.
    type Audio: Audio;
    /// Joypad interface.
    type Joypad: Joypad;
    /// Serial interface.
    type Serial: Serial;
    /// Video interface.
    type Video: Video;
}

/// Audio API.
mod audio {
    /// Audio interface.
    pub trait Audio {}
}

/// Joypad API.
mod joypad {
    pub use crate::core::dmg::Button;

    /// Joypad interface.
    pub trait Joypad {}
}

/// Serial API.
mod serial {
    use std::io::{self, Read, Write};

    /// Serial interface.
    pub trait Serial {
        /// Receive from the emulator.
        ///
        /// Provides a buffer that, when read, receives data sent by the
        /// emulator over the serial interface.
        ///
        /// Returns the number of bytes read by the frontend.
        ///
        /// # Errors
        ///
        /// Returns an error if the underlying buffer could not be read from.
        fn recv(buf: impl Read) -> io::Result<usize>;

        /// Transmit to the emulator.
        ///
        /// Provides a buffer that, when written, forwards data to the emulator
        /// over the serial interface.
        ///
        /// Returns the number of bytes written by the frontend.
        ///
        /// # Errors
        ///
        /// Returns an error if the underlying buffer could not be written to.
        fn send(buf: impl Write) -> io::Result<usize>;
    }
}

/// Video API.
mod video {
    pub use crate::api::pal::Color;
    pub use crate::core::dmg::ppu::Color as Pixel;
    pub use crate::core::dmg::{Screen, SCREEN as SIZE};

    /// Video interface.
    pub trait Video {
        fn draw(&mut self, lcd: &Screen);
    }
}
