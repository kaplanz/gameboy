use minifb::{Scale, Window, WindowOptions};

#[derive(Debug)]
pub struct View {
    pub tdat: Window,
    pub map1: Window,
    pub map2: Window,
}

impl View {
    pub fn new(opts: WindowOptions) -> Self {
        Self {
            tdat: Window::new("Tile Data", 16 * 8, 24 * 8, opts).unwrap(),
            map1: Window::new(
                "Tile Map 1",
                32 * 8,
                32 * 8,
                WindowOptions {
                    scale: Scale::X1,
                    ..opts
                },
            )
            .unwrap(),
            map2: Window::new(
                "Tile Map 2",
                32 * 8,
                32 * 8,
                WindowOptions {
                    scale: Scale::X1,
                    ..opts
                },
            )
            .unwrap(),
        }
    }
}
