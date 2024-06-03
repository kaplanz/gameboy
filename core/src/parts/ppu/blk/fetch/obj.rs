use rugby_arch::{Block, Byte};

use super::meta::{Layer, Row, Sprite};
use super::{Fifo, Lcdc, Ppu, Step};

/// Sprite fetcher.
#[derive(Clone, Debug)]
pub struct Fetcher {
    /// Sprite FIFO.
    pub fifo: Fifo,
    /// Fetcher step.
    pub step: Step,
}

impl Fetcher {
    /// Graphics layer.
    const LAYER: Layer = Layer::Sprite;
}

impl Default for Fetcher {
    fn default() -> Self {
        Self {
            fifo: Fifo::default(),
            step: Step::Fetch,
        }
    }
}

impl Fetcher {
    /// Executes a cycle of the fetcher.
    pub fn exec(&mut self, ppu: &mut Ppu, obj: &Sprite) {
        self.step = match self.step {
            Step::Fetch => exec::fetch(ppu, obj),
            Step::Read0 { tdat } => exec::read0(ppu, tdat),
            Step::Read1 { tdat, data } => exec::read1(ppu, tdat, data),
            Step::Push { data } => exec::push(self, data, obj),
        }
    }
}

impl Block for Fetcher {
    fn reset(&mut self) {
        std::mem::take(&mut self.fifo);
        std::mem::take(&mut self.step);
    }
}

/// Execution steps.
pub(super) mod exec {
    use log::trace;
    use rugby_arch::reg::Register;

    use super::{Byte, Fetcher, Lcdc, Ppu, Row, Sprite, Step};

    /// Executes fetch tile step.
    pub fn fetch(ppu: &Ppu, obj: &Sprite) -> Step {
        // Read the tile number from the tilemap
        let tnum = {
            // Check if the sprite is tall
            if ppu.lcdc(Lcdc::ObjSize) {
                // Tall (8x16) sprites span two tiles; must check if flipped
                let upper = {
                    let xpos = obj.ypos;
                    let line = ppu.reg.ly.load().saturating_add(16);
                    (xpos..xpos + 8).contains(&line)
                };
                if upper ^ obj.attr.yflip {
                    obj.tnum & 0b1111_1110 // use upper tile
                } else {
                    obj.tnum | 0b0000_0001 // use lower tile
                }
            } else {
                // Short (8x8) sprites only span a single tile
                obj.tnum
            }
        };
        trace!("used tile index: sprite.tnum -> #{tnum}");
        // Calculate the tile data address
        let mut tdat = ppu.tdat(Fetcher::LAYER, tnum);
        // Perform vertical flip
        if obj.attr.yflip {
            tdat ^= 0b0000_1110;
        }

        // Progress to next step
        Step::Read0 { tdat }
    }

    pub use super::super::bgw::exec::{read0, read1};

    /// Executes push to FIFO.
    pub fn push(fetch: &mut Fetcher, data: [Byte; 2], obj: &Sprite) -> Step {
        // Decode pixel row from bytes
        let mut row = Row::from(data);
        // Perform horizontal flip
        if obj.attr.xflip {
            row.xflip();
        }

        // Push to the FIFO
        //
        // NOTE: Some pixels may be discarded when FIFO is non-empty.
        trace!("pushed row of pixels: {row:?}");
        fetch.fifo.push(row, obj.meta());
        // Restart from beginning
        Step::Fetch
    }
}
