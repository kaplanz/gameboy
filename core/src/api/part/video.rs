//! Video API.

/// Video interface.
pub trait Video {
    /// Video output resolution.
    const SIZE: Aspect;

    /// Pixel data.
    type Pixel: Pixel;

    /// Checks for the start of vertical sync.
    ///
    /// Signals that the frame is ready to be rendered.
    #[must_use]
    fn vsync(&self) -> bool;

    /// Gets the current video framebuffer.
    ///
    /// If the frame is still being drawn, incomplete data may be yielded.
    ///
    /// # Warning
    ///
    /// Wait for [vsync](Video::vsync) before using the framebuffer to ensure it
    /// contains valid data.
    #[must_use]
    fn frame(&self) -> &[Self::Pixel];
}

/// Video aspect ratio.
#[derive(Debug)]
pub struct Aspect {
    /// Width in pixels.
    pub wd: u16,
    /// Height in pixels.
    pub ht: u16,
}

impl Aspect {
    /// Depth in pixels.
    #[must_use]
    pub const fn depth(&self) -> usize {
        (self.wd as usize).saturating_mul(self.ht as usize)
    }
}

/// Pixel color data.
pub trait Pixel: Copy + Default {}

/// Video framebuffer.
pub type Frame<P, const D: usize> = [P; D];
