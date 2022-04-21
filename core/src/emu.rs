use crate::io::joypad::Input;

/// Emulator interface.
///
/// Defines the common interface for emulators to handle I/O and other
/// communication with the user.
pub trait Emulator {
    /// Joypad button input.
    type Button: Input;
    /// Screen specification.
    type Screen;

    /// Sends currently pressed buttons.
    fn send(&mut self, btns: Vec<Self::Button>);

    /// Redraws the screen using the provided `callback` function.
    fn redraw(&self, callback: impl FnMut(&Self::Screen));
}
