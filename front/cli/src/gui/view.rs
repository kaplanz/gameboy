use super::{Error, Window};

#[derive(Debug)]
pub struct View {
    pub tdat: Window,
    pub map1: Window,
    pub map2: Window,
}

impl View {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            tdat: Window::new("Tile Data", 16 * 8, 24 * 8)?,
            map1: Window::new("Tile Map 1", 32 * 8, 32 * 8)?,
            map2: Window::new("Tile Map 2", 32 * 8, 32 * 8)?,
        })
    }
}
