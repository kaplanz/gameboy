use rugby_arch::reg::Register;
use rugby_arch::{Byte, Word};

use super::fifo::Fifo;
use super::meta::{self, Layer};
use super::{Lcdc, Ppu};

mod bgw;
mod obj;

pub use bgw::Fetcher as Background;
pub use obj::Fetcher as Sprite;

/// Fetcher step.
#[derive(Clone, Debug, Default)]
pub enum Step {
    /// Fetch tile number.
    #[default]
    Fetch,
    /// Read tile data (low).
    Read0 { tdat: Word },
    /// Read tile data (high).
    Read1 { tdat: Word, data: Byte },
    /// Push pixels to FIFO.
    Push { data: [Byte; 2] },
}

impl Ppu {
    /// Gets the tile address base for the configured addressing mode.
    #[inline]
    pub(crate) fn base(&self, layer: Layer) -> Word {
        match layer {
            Layer::Background | Layer::Window => {
                if self.lcdc(Lcdc::BgWinData) {
                    0x0000
                } else {
                    0x1000
                }
            }
            Layer::Sprite => 0x0000,
        }
    }

    /// Calculates a tile index for the configured addressing mode.
    #[inline]
    pub(crate) fn tidx(&self, layer: Layer, tnum: Byte) -> Word {
        let base = self.base(layer);
        let tnum = match layer {
            Layer::Background | Layer::Window => {
                if self.lcdc(Lcdc::BgWinData) {
                    Word::from(tnum)
                } else {
                    tnum as i8 as Word
                }
            }
            Layer::Sprite => Word::from(tnum),
        };
        base.wrapping_add(tnum << 4)
    }

    /// Calculates the tile offset for a given layer.
    #[inline]
    pub(crate) fn toff(&self, layer: Layer) -> Word {
        Word::from(match layer {
            Layer::Background | Layer::Sprite => {
                self.reg.ly.load().wrapping_add(self.reg.scy.load())
            }
            Layer::Window => self.etc.ywin,
        }) % 8
    }

    /// Calculates the tile data address from a tile number using the configured
    /// addressing mode.
    #[inline]
    pub(crate) fn tdat(&self, layer: Layer, tnum: Byte) -> Word {
        let tidx = self.tidx(layer, tnum);
        let toff = self.toff(layer) * 2;
        tidx | toff
    }
}
