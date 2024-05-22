//! Video API.

/// Video support.
pub trait Support {
    /// Video interface.
    type Video: Video;

    /// Gets the core's video.
    #[must_use]
    fn video(&self) -> &Self::Video;

    /// Mutably gets the core's video.
    #[must_use]
    fn video_mut(&mut self) -> &mut Self::Video;
}

/// Video interface.
pub trait Video {
    /// Video output resolution.
    const SIZE: Aspect;

    /// Pixel data.
    type Pixel: Pixel;

    /// Checks for the vertical sync.
    ///
    /// Signals that the frame is ready to be rendered.
    #[must_use]
    fn vsync(&self) -> bool;

    /// Gets the current video framebuffer.
    ///
    /// # Note
    ///
    /// If the frame is still being drawn, incomplete data may be yielded. This
    /// should always be checked first with [`Video::ready`].
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

/// Pixel data representation.
pub trait Pixel: Copy + Default {}

/// Framebuffer memory model.
pub type Frame<P, const D: usize> = [P; D];
