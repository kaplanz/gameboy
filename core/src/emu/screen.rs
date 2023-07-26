//! Screen interface.

use std::fmt::Debug;

/// Specified screen dimensions.
#[derive(Debug)]
pub struct Spec {
    pub width: usize,
    pub height: usize,
}

impl Spec {
    pub const fn depth(&self) -> usize {
        self.width.saturating_mul(self.height)
    }
}
