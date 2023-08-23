//! Common emulation interfaces.

use remus::Machine;

mod screen;

pub use screen::Screen;

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
