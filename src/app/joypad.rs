//! Joypad API.

use crate::emu::part::joypad::{Event, Input};

/// Joypad interface.
pub trait Joypad {
    type Button: Input;

    /// Polls joypad input for the emulator.
    ///
    /// The produced vector contains an ordered list of all input events that
    /// occured since input was last polled.
    fn input(&mut self) -> Vec<Event<Self::Button>>;
}
