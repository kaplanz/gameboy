use remus::Device;

use super::fifo::Fifo;
use super::pixel::{Meta, Palette};
use super::sprite::Sprite;
use super::tile::Row;
use super::{Lcdc, Ppu};

#[derive(Debug, Default)]
pub struct Fetch {
    busy: bool,
    xidx: u8,
    stage: Stage,
}

impl Fetch {
    #[must_use]
    pub fn stage(&self) -> &Stage {
        &self.stage
    }

    pub fn exec(&mut self, fifo: &mut Fifo, ppu: &mut Ppu, loc: Location, sprite: Option<Sprite>) {
        // NOTE: Some stages of the fetch take 2 dots to complete, so only
        //       execute when we're not already busy from the current stage.
        if self.busy {
            self.busy = false;
        } else {
            self.stage = std::mem::take(&mut self.stage).exec(self, fifo, ppu, loc, sprite);
        }
    }

    fn tnum(&self, ppu: &Ppu, loc: Location) -> u16 {
        use Location::{Background, Sprite, Window};

        // For sprites, we don't need this
        if loc == Sprite {
            unreachable!("Sprites have no tile number")
        }

        // Extract scanline info
        let lcdc = **ppu.ctl.lcdc.borrow();
        let scy = **ppu.ctl.scy.borrow();
        let scx = **ppu.ctl.scx.borrow();
        let ly = **ppu.ctl.ly.borrow();

        // Determine the tile base
        let base = match loc {
            Background => {
                let bgmap = Lcdc::BgMap.get(lcdc);
                [0x1800, 0x1c00][bgmap as usize]
            }
            Window => {
                let winmap = Lcdc::WinMap.get(lcdc);
                [0x1800, 0x1c00][winmap as usize]
            }
            Sprite => unreachable!(),
        };

        // Calculate the tile offset
        let row: u8;
        let col: u8;
        match loc {
            Background => {
                row = ly.wrapping_add(scy) / 8;
                col = (self.xidx + (scx / 8)) & 0x1f;
            }
            Window => {
                row = ppu.winln / 8;
                col = self.xidx;
            }
            Sprite => unreachable!(),
        }
        let offset = (32 * row as u16) + col as u16;

        // Calculate the tile number
        base + offset
    }

    fn addr(ppu: &Ppu, loc: Location, tidx: u8) -> u16 {
        use Location::{Background, Sprite, Window};

        // Extract scanline info
        let lcdc = **ppu.ctl.lcdc.borrow();
        let scy = **ppu.ctl.scy.borrow();
        let ly = **ppu.ctl.ly.borrow();

        // Calculate the y-offset within the tile
        let yoff = match loc {
            Background | Sprite => ly.wrapping_add(scy) % 8,
            Window => ppu.winln % 8,
        };

        // Calculate the tile data address
        match loc {
            Background | Window => {
                if Lcdc::BgWinData.get(lcdc) {
                    let base = 0x0000;
                    let tidx = tidx as u16;
                    let offset = (16 * tidx) + (2 * yoff) as u16;
                    base + offset
                } else {
                    let base = 0x1000;
                    let tidx = tidx as i8 as i16;
                    let offset = (16 * tidx) + (2 * yoff) as i16;
                    (base + offset) as u16
                }
            }
            Sprite => {
                let base = 0x0000;
                let tidx = tidx as u16;
                let offset = (16 * tidx) + (2 * yoff) as u16;
                base + offset
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Location {
    #[default]
    Background,
    Window,
    Sprite,
}

#[derive(Debug, Default)]
pub enum Stage {
    #[default]
    ReadTile,
    ReadData0 {
        addr: u16,
    },
    ReadData1 {
        addr: u16,
        data0: u8,
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
                let tidx = if let Some(obj) = sprite {
                    // Check if the sprite is tall (8x16)
                    let lcdc = **ppu.ctl.lcdc.borrow();
                    let tall = Lcdc::ObjSize.get(lcdc);
                    if tall {
                        // Determine if we're fetching the top or bottom
                        // tile of the tall sprite.
                        let ly = **ppu.ctl.ly.borrow();
                        let top = (obj.ypos..obj.ypos + 8).contains(&(ly + 16));
                        if top ^ obj.yflip {
                            obj.idx & 0b1111_1110
                        } else {
                            obj.idx | 0b0000_0001
                        }
                    } else {
                        // Short (8x8) sprites only span a single tile
                        obj.idx
                    }
                } else {
                    // Calculate the tile number
                    // NOTE: How this is calculated depends on the type of the tile
                    //       being fetched.
                    let tnum = fetch.tnum(ppu, loc);
                    ppu.vram.borrow().read(tnum as usize)
                };

                // NOTE: We can calculate the tile data address in advance. This
                //       is more efficient than doing so once each data read.
                let addr = Fetch::addr(ppu, loc, tidx);

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
                let data0 = ppu.vram.borrow().read(addr as usize);

                // Progress to next stage
                let addr = addr + 1;
                Stage::ReadData1 { addr, data0 }
            }
            Stage::ReadData1 { addr, data0 } => {
                // Fetch the seocnd byte of the tile
                let data1 = ppu.vram.borrow().read(addr as usize);

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
