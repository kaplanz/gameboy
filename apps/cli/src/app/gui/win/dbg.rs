use super::{Aspect, Result, Window};

/// Debug memory region.
#[derive(Clone, Copy, Debug)]
pub enum Region {
    /// Tile data.
    Tile,
    /// Tile map 1.
    Map1,
    /// Tile map 2.
    Map2,
}

impl Region {
    /// Gets the region's title.
    fn title(self) -> &'static str {
        match self {
            Region::Tile => "Tile Data",
            Region::Map1 => "Tile Map 1",
            Region::Map2 => "Tile Map 2",
        }
    }

    /// Gets the region's aspect.
    fn aspect(self) -> Aspect {
        match self {
            Region::Tile => Aspect {
                wd: 16 * 8,
                ht: 24 * 8,
            },
            Region::Map1 | Region::Map2 => Aspect {
                wd: 32 * 8,
                ht: 24 * 8,
            },
        }
    }
}

#[derive(Debug, Default)]
pub struct Debug {
    /// Tile data.
    tile: Option<Window>,
    /// Tile map 1.
    map1: Option<Window>,
    /// Tile map 2.
    map2: Option<Window>,
}

impl Debug {
    /// Constructs a new `Debug`.
    #[allow(unused)]
    pub fn new() -> Result<Self> {
        let mut this = Self::default();
        this.all()?;
        Ok(this)
    }

    /// Opens all debug windows.
    pub fn all(&mut self) -> Result<()> {
        self.open(Region::Tile)?;
        self.open(Region::Map1)?;
        self.open(Region::Map2)?;
        Ok(())
    }

    /// Opens a window for the selected [`Region`].
    ///
    /// # Note
    ///
    /// If the selected window is already open, this function is a no-op.
    pub fn open(&mut self, sel: Region) -> Result<()> {
        // Get the selected window
        let win = match sel {
            Region::Tile => &mut self.tile,
            Region::Map1 => &mut self.map1,
            Region::Map2 => &mut self.map2,
        };
        // No-op if window exists.
        let None = win else { return Ok(()) };
        // Create the selected window.
        *win = Some(Window::new(sel.title(), sel.aspect())?);

        Ok(())
    }

    /// Gets the selected window.
    #[allow(unused)]
    pub fn get(&self, sel: Region) -> Option<&Window> {
        match sel {
            Region::Tile => self.tile.as_ref(),
            Region::Map1 => self.map1.as_ref(),
            Region::Map2 => self.map2.as_ref(),
        }
    }

    /// Mutably gets the selected window.
    #[allow(unused)]
    pub fn get_mut(&mut self, sel: Region) -> Option<&mut Window> {
        match sel {
            Region::Tile => self.tile.as_mut(),
            Region::Map1 => self.map1.as_mut(),
            Region::Map2 => self.map2.as_mut(),
        }
    }
}
