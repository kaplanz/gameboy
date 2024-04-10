//! Joypad API.

use crate::emu::joypad::{Event, Input};

/// Joypad interface.
pub trait Joypad {
    type Button: Input;

    fn input(&mut self) -> Vec<Event<Self::Button>>;
}
