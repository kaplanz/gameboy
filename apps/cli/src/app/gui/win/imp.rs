//! Window implementation.

use std::collections::HashSet;

pub use minifb::{Error, Key};
use minifb::{Result, Scale, ScaleMode, WindowOptions};
use rugby::emu::joypad::{Event, State};

/// Color value of an individual pixel.
pub type Pixel = u32;

/// Window aspect ratio.
#[derive(Debug)]
pub struct Aspect {
    /// Width in pixels.
    pub wd: usize,
    /// Height in pixels.
    pub ht: usize,
}

impl From<(usize, usize)> for Aspect {
    fn from((wd, ht): (usize, usize)) -> Self {
        Self { wd, ht }
    }
}

/// Application window.
#[derive(Debug)]
pub struct Window {
    /// OS window handle.
    win: Box<minifb::Window>,
    /// Logical aspect ratio.
    asp: Aspect,
}

impl Window {
    /// Window options.
    const OPTIONS: WindowOptions = WindowOptions {
        borderless: false,
        title: true,
        resize: true,
        scale: Scale::X2,
        scale_mode: ScaleMode::AspectRatioStretch,
        topmost: false,
        transparency: false,
        none: false,
    };

    /// Constructs a new `Window`.
    pub fn new(title: &str, asp: Aspect) -> Result<Self> {
        Ok(Self {
            win: Box::new(minifb::Window::new(title, asp.wd, asp.ht, Self::OPTIONS)?),
            asp,
        })
    }

    /// Checks if the window is open.
    pub fn is_open(&self) -> bool {
        self.win.is_open()
    }

    /// Sets the window's title.
    pub fn set_title(&mut self, title: &str) {
        self.win.set_title(title);
    }

    /// Redraws the window's contents using the provided buffer.
    pub fn redraw(&mut self, buf: &[Pixel]) -> Result<()> {
        self.win.update_with_buffer(buf, self.asp.wd, self.asp.ht)
    }

    /// Update internal key events.
    pub fn keys(&self) -> Vec<Event<Key>> {
        // Get keys
        let dn: HashSet<_> = self
            .win
            .get_keys_pressed(minifb::KeyRepeat::No)
            .into_iter()
            .collect();
        let up: HashSet<_> = self.win.get_keys_released().into_iter().collect();
        // Dedup keys
        let (dn, up) = (dn.difference(&up), up.difference(&dn));
        // Pair events
        let dn = dn.copied().map(|key| (key, State::Dn).into());
        let up = up.copied().map(|key| (key, State::Up).into());
        // Collect all
        dn.chain(up).collect()
    }
}
