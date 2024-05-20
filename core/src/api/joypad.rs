//! Joypad API.

use std::hash::Hash;

/// Joypad support.
pub trait Support {
    /// Joypad interface.
    type Joypad: Joypad;

    /// Gets the core's joypad.
    #[must_use]
    fn joypad(&self) -> &Self::Joypad;

    /// Mutably gets the core's joypad.
    #[must_use]
    fn joypad_mut(&mut self) -> &mut Self::Joypad;
}

/// Joypad interface.
pub trait Joypad {
    /// User input.
    type Button: Input;

    /// Receives user input events, forwarding to the core.
    fn recv(&mut self, events: impl IntoIterator<Item = Event<Self::Button>>);
}

/// Joypad input.
pub trait Input: Copy + Eq + Hash {}

/// Jopypad event.
#[derive(Clone, Copy, Debug)]
pub struct Event<I>
where
    I: Copy + Eq + Hash,
{
    /// Input identifier.
    pub input: I,
    /// Input state.
    pub state: State,
}

impl<I: Copy + Eq + Hash> From<(I, State)> for Event<I> {
    fn from(value: (I, State)) -> Self {
        let (input, state) = value;
        Event { input, state }
    }
}

/// Joypad button state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    /// Button pressed.
    Dn,
    /// Button released.
    Up,
}
