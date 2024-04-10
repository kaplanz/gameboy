use std::fmt::Display;

use remus::Cell;

use super::blk::Pipeline;
use super::scan::Scan;
use super::sprite::Sprite;
use super::{Mode, Ppu, LCD};

/// Pixel drawing interval.
#[derive(Clone, Debug, Default)]
pub struct Draw {
    pub(crate) pipe: Pipeline,
    pub(crate) objs: Vec<Sprite>,
}

impl Draw {
    pub fn setup(&mut self, ppu: &mut Ppu) {
        // Set up the pipeline
        let scx = ppu.file.scx.load();
        self.pipe.set_discard(scx);
    }

    pub fn exec(mut self, ppu: &mut Ppu) -> Mode {
        // Execute the next fetch cycle
        self.pipe.fetch(ppu, &self.objs);

        // Extract x-position
        let mut xpos: u16 = self.pipe.xpos().into();
        // If we have a pixel to draw, draw it
        if let Some(pixel) = self.pipe.shift(ppu) {
            // Extract y-position
            let ypos: u16 = ppu.file.ly.load().into();

            // Calculate pixel index on screen
            let idx = (ypos * LCD.wd) + xpos;
            // Determine this pixel's color (according to its palette)
            let color = ppu.color(&pixel);
            // Write pixel into the framebuffer
            ppu.buf[idx as usize] = color;

            // Update x-position
            xpos = self.pipe.xpos().into();
        }

        // Move to next dot
        ppu.dot += 1;

        // Determine next mode
        if xpos < LCD.wd {
            // Continue to next pixel
            Mode::Draw(self)
        } else {
            // Increment window internal line counter
            if self.pipe.within_win() {
                ppu.winln += 1;
            }
            // Enter hblank
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
