use rugby_arch::mem::Memory;
use rugby_arch::reg::Register;
use rugby_arch::{Byte, Word};

use super::fifo::Fifo;
use super::pixel::{Meta, Palette};
use super::sprite::Sprite;
use super::tile::Row;
use super::{Lcdc, Ppu};

/// PPU's pixel fetcher.
#[derive(Clone, Debug, Default)]
pub struct Fetch {
    busy: bool,
    xidx: Byte,
    stage: Stage,
}

impl Fetch {
    /// Gets the fetcher's stage.
    #[must_use]
    pub fn stage(&self) -> &Stage {
        &self.stage
    }

    /// Executes a stage of the fetcher.
    pub fn exec(&mut self, fifo: &mut Fifo, ppu: &mut Ppu, loc: Location, sprite: Option<Sprite>) {
        // NOTE: Some stages of the fetch take 2 dots to complete, so only
        //       execute when we're not already busy from the current stage.
        if self.busy {
            self.busy = false;
        } else {
            self.stage = std::mem::take(&mut self.stage).exec(self, fifo, ppu, loc, sprite);
        }
    }

    /// Calculates the tile number for this fetcher.
    pub fn cpos(&self, ppu: &Ppu, loc: Location) -> Word {
        use Location::{Background, Sprite, Window};

        // For sprites, we don't need this
        if loc == Sprite {
            unreachable!("Sprites have no tile number")
        }

        // Extract scanline info
        let lcdc = ppu.reg.lcdc.load();
        let scy = ppu.reg.scy.load();
        let scx = ppu.reg.scx.load();
        let ly = ppu.reg.ly.load();

        // Determine the tile base
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
                row = ly.wrapping_add(scy) / 8;
                col = (self.xidx + (scx / 8)) & 0x1f;
            }
            Window => {
                row = ppu.etc.win / 8;
                col = self.xidx;
            }
            Sprite => unreachable!(),
        }
        let offset = (32 * row as Word) + col as Word;

        // Calculate the tile number
        base + offset
    }

    /// Calculates the address for a given tile number and location.
    pub fn addr(ppu: &Ppu, loc: Location, tnum: Byte) -> Word {
        use Location::{Background, Sprite, Window};

        // Extract scanline info
        let lcdc = ppu.reg.lcdc.load();
        let scy = ppu.reg.scy.load();
        let ly = ppu.reg.ly.load();

        // Calculate the y-offset within the tile
        let yoff = match loc {
            Background | Sprite => ly.wrapping_add(scy),
            Window => ppu.etc.win,
        } % 8;

        // Calculate the tile data address
        let bgwin = Lcdc::BgWinData.get(&lcdc);
        let tidx = Self::tidx(loc, bgwin, tnum);
        let toff = (2 * yoff) as Word;
        tidx | toff
    }

    /// Calculates a tile number into a tile map index according to the selected
    /// addressing mode.
    pub fn tidx(loc: Location, bgwin: bool, tnum: Byte) -> Word {
        use Location::{Background, Sprite, Window};
        let (base, tnum): (Word, Word) = match (loc, bgwin) {
            (Background | Window, true) | (Sprite, _) => (0x0000, Word::from(tnum)),
            (Background | Window, false) => (0x1000, tnum as i8 as Word),
        };
        base.wrapping_add(tnum << 4)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Location {
    #[default]
    Background,
    Window,
    Sprite,
}

#[derive(Clone, Debug, Default)]
pub enum Stage {
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

impl Stage {
    fn exec(
        self,
        fetch: &mut Fetch,
        fifo: &mut Fifo,
        ppu: &mut Ppu,
        loc: Location,
        sprite: Option<Sprite>,
    ) -> Self {
        match self {
            Stage::ReadTile => {
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

                // Progress to next stage
                Stage::ReadData0 { addr }
            }
            Stage::ReadData0 { mut addr } => {
                // Perform y-flip on sprites
                if let Some(obj) = sprite {
                    if obj.yflip {
                        super::sprite::Sprite::yflip(&mut addr);
                    }
                }

                // Fetch the first byte of the tile
                let data0 = ppu.mem.vram.read(addr).unwrap();

                // Progress to next stage
                let addr = addr + 1;
                Stage::ReadData1 { addr, data0 }
            }
            Stage::ReadData1 { addr, data0 } => {
                // Fetch the seocnd byte of the tile
                let data1 = ppu.mem.vram.read(addr).unwrap();

                // Decode pixels from data
                let row = Row::from([data0, data1]);
                let meta = match sprite {
                    Some(obj) => Sprite::meta(obj),
                    None => Meta {
                        pal: Palette::BgWin,
                        bgp: false,
                    }, // Background/Window
                };

                // Progress to next stage
                Stage::Push(row, meta)
            }
            Stage::Push(mut row, meta) => {
                // Perform x-flip on sprites
                if let Some(obj) = sprite {
                    if obj.xflip {
                        row.xflip();
                    }
                }

                // Attempt to push row to FIFO
                match fifo.try_append(row, meta) {
                    Ok(()) => {
                        // Move fetch to next x-position
                        fetch.xidx += 1;
                        // Restart fetch
                        Stage::ReadTile
                    }
                    Err((row, meta)) => {
                        // Try again next cycle
                        Stage::Push(row, meta)
                    }
                }
            }
        }
    }
}
