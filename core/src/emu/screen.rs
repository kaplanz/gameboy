//! Screen interface.

use std::fmt::Debug;

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
