//! Video API.

use crate::emu::video::Pixel;

/// Video interface.
pub trait Video {
    type Pixel: Pixel;

    fn draw(&mut self, frame: &[Self::Pixel]);
}
