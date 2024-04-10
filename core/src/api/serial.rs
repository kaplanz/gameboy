//! Serial API.

use std::io::{BufRead, Write};

/// Serial support.
pub trait Support {
    /// Serial interface.
    type Serial: Serial;

    /// Gets the core's serial.
    #[must_use]
    fn serial(&self) -> &Self::Serial;

    /// Mutably gets the core's serial.
    #[must_use]
    fn serial_mut(&mut self) -> &mut Self::Serial;
}

/// Serial interface.
pub trait Serial {
    /// Gets the external serial receiver.
    ///
    /// Reading from the receiver yields data produced by the core.
    #[must_use]
    fn rx(&mut self) -> &mut impl BufRead;

    /// Gets the external serial transmitter.
    ///
    /// Writing to the transmitter forwards data to the core.
    #[must_use]
    fn tx(&mut self) -> &mut impl Write;
}
