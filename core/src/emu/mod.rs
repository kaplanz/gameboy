//! Common emulation interfaces.

use remus::Machine;

pub mod joypad;
pub mod screen;

/// Emulator interface.
///
/// Defines the interface between an emulator core and the frontend.
pub trait Emulator: Machine {
    /// Joypad input.
    type Input;

    /// Screen controller.
    type Screen;

    /// Sends currently pressed keys.
    fn send(&mut self, keys: &[Self::Input]);

    /// Redraws the screen using the current state.
    fn redraw(&self, callback: impl FnMut(&Self::Screen));
}
