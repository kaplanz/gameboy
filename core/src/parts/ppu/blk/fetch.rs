use log::trace;
use rugby_arch::mem::Memory;
use rugby_arch::reg::Register;
use rugby_arch::{Byte, Word};

use super::fifo::Fifo;
use super::pixel::{Meta, Palette};
use super::sprite::Sprite;
use super::tile::Row;
use super::{Lcdc, Ppu};

/// Pixel fetcher.
#[derive(Clone, Debug, Default)]
pub struct Fetch {
    /// LCD X-coordinate.
    pub lx: Byte,
    /// Fetch step.
    pub step: Step,
}

impl Fetch {
    /// Executes a step of the fetcher.
    pub fn exec(&mut self, fifo: &mut Fifo, ppu: &mut Ppu, loc: Layer, sprite: Option<Sprite>) {
        self.step = std::mem::take(&mut self.step).exec(self, fifo, ppu, loc, sprite);
    }

    /// Calculates the tile number for this fetcher.
    pub fn cpos(&self, ppu: &Ppu, loc: Layer) -> Word {
        use Layer::{Background, Sprite, Window};

        // For sprites, we don't need this
        if loc == Sprite {
            unreachable!("Sprites have no tile number")
        }

        // Determine the tile base
        let lcdc = ppu.reg.lcdc.load();
        let base = match loc {
            Background => {
                let bgmap = Lcdc::BgMap.get(&lcdc);
                [0x1800, 0x1c00][bgmap as usize]
            }
            Window => {
                let winmap = Lcdc::WinMap.get(&lcdc);
                [0x1800, 0x1c00][winmap as usize]
            }
            Sprite => unreachable!(),
        };

        // Calculate the tile offset
        let row: Byte;
        let col: Byte;
        match loc {
            Background => {
                row = ppu.reg.ly.load().wrapping_add(ppu.reg.scy.load()) / 8;
                col = 0x1f & (self.lx + (ppu.reg.scx.load() / 8));
            }
            Window => {
                row = ppu.etc.win / 8;
                col = self.lx;
            }
            Sprite => unreachable!(),
        }
        let offset = (32 * row as Word) + col as Word;

        // Calculate the tile number
        base + offset
    }

    /// Calculates the address for a given tile number and location.
    pub fn addr(ppu: &Ppu, loc: Layer, tnum: Byte) -> Word {
        use Layer::{Background, Sprite, Window};

        // Calculate the y-offset within the tile
        let yoff = match loc {
            Background | Sprite => ppu.reg.ly.load().wrapping_add(ppu.reg.scy.load()),
            Window => ppu.etc.win,
        } % 8;

        // Calculate the tile data address
        let lcdc = ppu.reg.lcdc.load();
        let bgwin = Lcdc::BgWinData.get(&lcdc);
        let tidx = Self::tidx(loc, bgwin, tnum);
        let toff = (2 * yoff) as Word;
        tidx | toff
    }

    /// Calculates a tile number into a tile map index according to the selected
    /// addressing mode.
    pub fn tidx(loc: Layer, bgwin: bool, tnum: Byte) -> Word {
        use Layer::{Background, Sprite, Window};
        let (base, tnum): (Word, Word) = match (loc, bgwin) {
            (Background | Window, true) | (Sprite, _) => (0x0000, Word::from(tnum)),
            (Background | Window, false) => (0x1000, tnum as i8 as Word),
        };
        base.wrapping_add(tnum << 4)
    }
}

/// Fetch layer.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Layer {
    #[default]
    Background,
    Window,
    Sprite,
}

/// Fetch step.
#[derive(Clone, Debug, Default)]
pub enum Step {
    #[default]
    ReadTile,
    ReadData0 {
        addr: Word,
    },
    ReadData1 {
        addr: Word,
        data0: Byte,
    },
    Push(Row, Meta),
}

impl Step {
    fn exec(
        self,
        fetch: &mut Fetch,
        fifo: &mut Fifo,
        ppu: &mut Ppu,
        loc: Layer,
        sprite: Option<Sprite>,
    ) -> Self {
        match self {
            Step::ReadTile => {
                // Fetch the tile number's index
                let tnum = if let Some(obj) = sprite {
                    // Check if the sprite is tall (8x16)
                    let lcdc = ppu.reg.lcdc.load();
                    let tall = Lcdc::ObjSize.get(&lcdc);
                    if tall {
                        // Determine if we're fetching the top or bottom
                        // tile of the tall sprite.
                        let ly = ppu.reg.ly.load();
                        let top = (obj.ypos..obj.ypos + 8).contains(&(ly + 16));
                        if top ^ obj.yflip {
                            obj.tidx & 0b1111_1110
                        } else {
                            obj.tidx | 0b0000_0001
                        }
                    } else {
                        // Short (8x8) sprites only span a single tile
                        obj.tidx
                    }
                } else {
                    // Calculate the tile number
                    // NOTE: How this is calculated depends on the type of the tile
                    //       being fetched.
                    let cpos = fetch.cpos(ppu, loc);
                    ppu.mem.vram.read(cpos).unwrap()
                };

                // NOTE: We can calculate the tile data address in advance. This
                //       is more efficient than doing so once each data read.
                let addr = Fetch::addr(ppu, loc, tnum);
                trace!("fetched from address {addr:#06x} (tile #{tnum})");

                // Progress to next step
                Step::ReadData0 { addr }
            }
            Step::ReadData0 { mut addr } => {
                // Perform y-flip on sprites
                if let Some(obj) = sprite {
                    if obj.yflip {
                        super::sprite::Sprite::yflip(&mut addr);
                    }
                }

                // Fetch the first byte of the tile
                let data0 = ppu.mem.vram.read(addr).unwrap();
                trace!("read lower byte of data: {data0:#04x}");

                // Progress to next step
                let addr = addr + 1;
                Step::ReadData1 { addr, data0 }
            }
            Step::ReadData1 { addr, data0 } => {
                // Fetch the seocnd byte of the tile
                let data1 = ppu.mem.vram.read(addr).unwrap();
                trace!("read upper byte of data: {data1:#04x}");

                // Decode pixels from data
                let row = Row::from([data0, data1]);
                let meta = match sprite {
                    Some(obj) => Sprite::meta(obj),
                    None => Meta {
                        pal: Palette::BgWin,
                        bgp: false,
                    }, // Background/Window
                };

                // Progress to next step
                Step::Push(row, meta)
            }
            Step::Push(mut row, meta) => {
                // Perform x-flip on sprites
                if let Some(obj) = sprite {
                    if obj.xflip {
                        row.xflip();
                    }
                }

                // Attempt to push row to FIFO
                if fifo.push(&row, meta) {
                    trace!("pushed row of pixels: {row:?}");
                    // Move fetch to next x-coordinate
                    fetch.lx += 1;
                    // Restart fetch
                    Step::ReadTile
                } else {
                    trace!("cannot push to non-empty FIFO; will retry");
                    // Try again next cycle
                    Step::Push(row, meta)
                }
            }
        }
    }
}
