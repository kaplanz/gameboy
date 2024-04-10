//! Serial API.

use std::io::{self, Read, Write};

/// Serial interface.
pub trait Serial {
    /// Receive data from the emulator.
    ///
    /// Provides a buffer that, when read, receives data sent by the
    /// emulator over the serial interface; returns the number of bytes read by
    /// the frontend.
    ///
    /// # Errors
    ///
    /// This function will return an error when the underlying buffer could not
    /// be read from.
    fn recv(&mut self, buf: impl Read) -> io::Result<usize>;

    /// Transmit data to the emulator.
    ///
    /// Provides a buffer that, when written, forwards data to the emulator
    /// over the serial interface; returns the number of bytes written by the
    /// frontend.
    ///
    /// # Errors
    ///
    /// This function will return an error when the underlying buffer could not
    /// be written to.
    fn send(&mut self, buf: impl Write) -> io::Result<usize>;
}
