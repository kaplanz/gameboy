//! Serial API.

use std::io::{BufRead, Write};

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
