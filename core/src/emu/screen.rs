//! Screen primitives.

/// Screen dimensions.
#[derive(Debug)]
pub struct Screen {
    /// Width of the screen in pixels.
    pub width: usize,
    /// Height of the screen in pixels.
    pub height: usize,
}

impl Screen {
    /// Depth of the screen in pixels.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.width.saturating_mul(self.height)
    }
}
