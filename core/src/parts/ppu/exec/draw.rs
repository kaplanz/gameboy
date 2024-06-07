use log::{debug, trace};
use rugby_arch::reg::Register;

use super::ppu::blk::pipe::Pipeline;
use super::ppu::meta::{Layer, Palette, Pixel, Sprite};
use super::ppu::Color;
use super::scan::Scan;
use super::{Mode, Ppu, LCD};

/// Mode 3: Draw pixels.
#[derive(Clone, Debug, Default)]
pub struct Draw {
    /// Pixel pipeline.
    pub(super) pipe: Pipeline,
    /// Scanned objects.
    pub(super) objs: Vec<Sprite>,
}

impl Draw {
    pub fn exec(mut self, ppu: &mut Ppu) -> Mode {
        // Initialize discarded scroll pixels
        if !self.pipe.ready {
            self.pipe.scroll = ppu.reg.scx.load() % 8;
        }

        // Execute the next fetch cycle
        self.pipe.fetch(ppu, &self.objs);

        // If we have a pixel to draw, draw it
        if let Some(pixel) = self.pipe.shift(ppu) {
            // Fetch pixel coordinates
            let ly: u16 = ppu.reg.ly.load().into();
            let lx: u16 = self.pipe.lx.into();

            // Write pixel into the framebuffer
            let color = ppu.color(&pixel); // determine color
            let pidx = (ly * LCD.wd) + lx; // calculate index
            ppu.etc.buf[usize::from(pidx)] = color;
            trace!("wrote pixel: {color:?} -> (row: {ly}, col: {lx})");

            // Move to next pixel
            self.pipe.lx += 1;
        }

        // Determine next mode
        if u16::from(self.pipe.lx) < LCD.wd {
            // Continue to next pixel
            Mode::Draw(self)
        } else {
            // Increment window internal line counter
            if self.pipe.bgw.layer == Layer::Window {
                ppu.etc.ywin += 1;
            }
            // Enter hblank
            debug!("entered mode 0: hblank");
            Mode::HBlank(self.into())
        }
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

impl Ppu {
    /// Color a pixel using the current palette.
    pub(in super::super) fn color(&self, pixel: &Pixel) -> Color {
        // Load palette data
        let pal = match pixel.meta.pal {
            Palette::BgWin => self.reg.bgp.load(),
            Palette::Obp0 => self.reg.obp0.load(),
            Palette::Obp1 => self.reg.obp1.load(),
        };
        // Assign colors using palette
        #[allow(clippy::identity_op, unused_parens)]
        let col = Color::from(match pixel.col {
            Color::C0 => (0b0000_0011 & pal) >> 0,
            Color::C1 => (0b0000_1100 & pal) >> 2,
            Color::C2 => (0b0011_0000 & pal) >> 4,
            Color::C3 => (0b1100_0000 & pal) >> 6,
        });
        trace!(
            "transformed: {old:?} -> {col:?}, using: {reg:?} = {pal:#010b}",
            old = pixel.col,
            reg = pixel.meta.pal,
        );
        col
    }
}
