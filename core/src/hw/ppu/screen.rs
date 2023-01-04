use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use super::pixel::Color;
use crate::dmg::SCREEN;

/// Screen data.
#[derive(Debug)]
pub struct Screen([Color; SCREEN.depth()]);

impl Default for Screen {
    fn default() -> Self {
        Self([Color::default(); SCREEN.depth()])
    }
}

impl Deref for Screen {
    type Target = [Color];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<[Color; SCREEN.depth()]> for Screen {
    fn from(buf: [Color; SCREEN.depth()]) -> Self {
        Self(buf)
    }
}
