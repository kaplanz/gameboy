//! Audio API.

/// Audio support.
pub trait Support {
    /// Audio interface.
    type Audio: Audio;

    /// Gets the core's audio.
    #[must_use]
    fn audio(&self) -> &Self::Audio;

    /// Mutably gets the core's audio.
    #[must_use]
    fn audio_mut(&mut self) -> &mut Self::Audio;
}

/// Audio interface.
pub trait Audio {}
