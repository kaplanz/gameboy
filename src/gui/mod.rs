use minifb::{Key, Scale, ScaleMode, WindowOptions};
use thiserror::Error;

#[cfg(feature = "view")]
use self::view::View;

#[cfg(feature = "view")]
pub mod view;

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

#[derive(Debug)]
pub struct Gui {
    pub main: Window,
    #[cfg(feature = "view")]
    pub view: Option<View>,
}

impl Gui {
    /// Checks if the GUI is alive.
    pub fn alive(&self) -> bool {
        self.main.is_open()
    }
}

#[derive(Debug)]
pub struct Window {
    win: minifb::Window,
    wx: usize,
    hy: usize,
}

impl Window {
    /// Constructs a new `Window`.
    pub fn new(title: &str, width: usize, height: usize) -> Result<Self, Error> {
        Ok(Self {
            win: minifb::Window::new(title, width, height, OPTIONS)?,
            wx: width,
            hy: height,
        })
    }

    /// Gets the currently pressed keys.
    pub fn get_keys(&self) -> Vec<Key> {
        self.win.get_keys()
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
    pub fn redraw(&mut self, buf: &[u32]) -> Result<(), Error> {
        Ok(self.win.update_with_buffer(buf, self.wx, self.hy)?)
    }
}

/// A type specifying categories of [`Gui`] errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Window(#[from] minifb::Error),
}
