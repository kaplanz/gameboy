use std::fmt::Display;

use super::blk::Pipeline;
use super::sprite::Sprite;
use super::{Mode, Ppu, Scan, SCREEN};

#[derive(Debug, Default)]
pub struct Draw {
    pub(crate) pipe: Pipeline,
    pub(crate) objs: Vec<Sprite>,
}

impl Draw {
    pub fn setup(&mut self, ppu: &mut Ppu) {
        // Set up the pipeline
        let scx = **ppu.ctl.borrow().scx.borrow();
        self.pipe.set_discard(scx);
    }

    pub fn exec(mut self, ppu: &mut Ppu) -> Mode {
        // Execute the next fetch cycle
        self.pipe.fetch(ppu, &self.objs);

        // If we have a pixel to draw, draw it
        let mut xpos = self.pipe.xpos() as usize;
        if let Some(pixel) = self.pipe.shift(ppu) {
            // Calculate pixel index on screen
            let ypos = **ppu.ctl.borrow().ly.borrow() as usize;
            let idx = (ypos * SCREEN.width) + xpos;

            // Determine this pixel's color (according to its palette)
            let color = ppu.color(&pixel);

            // Write the pixel into the framebuffer
            ppu.lcd[idx] = color;

            // Retrieve updated x-position
            xpos = self.pipe.xpos() as usize;
        }

        // Move to the next dot
        ppu.dot += 1;

        // Either draw next pixel, or enter HBlank
        if xpos < SCREEN.width {
            Mode::Draw(self)
        } else {
            // Increment window internal line counter
            if self.pipe.was_at_win() {
                ppu.winln += 1;
            }
            Mode::HBlank(self.into())
        }
    }
}

impl Display for Draw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─────────────┐")?;
        writeln!(f, "│ {:^11} │", "Draw")?;
        writeln!(f, "├─────────────┤")?;
        writeln!(f, "│ Column: {:>3} │", self.pipe.xpos())?;
        writeln!(f, "│ Objects: {:>2} │", self.objs.len())?;
        write!(f, "└─────────────┘")
    }
}

impl From<Scan> for Draw {
    fn from(Scan { objs, .. }: Scan) -> Self {
        Self {
            objs,
            ..Default::default()
        }
    }
}
