//! Screen interface.

use std::fmt::Debug;
use std::ops::DerefMut;

/// Screen info.
#[derive(Debug)]
pub struct Info {
    pub width: usize,
    pub height: usize,
}

impl Info {
    pub const fn depth(&self) -> usize {
        self.width.saturating_mul(self.height)
    }
}

/// Screen interface.
pub trait Screen: Clone + Debug + DerefMut<Target = [Self::Pixel]> {
    /// Individual pixel.
    type Pixel;

    /// Redraws the screen using the provided `callback` function.
    fn redraw(&self);
}
