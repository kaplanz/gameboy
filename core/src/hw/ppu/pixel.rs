use std::ops::{Deref, DerefMut};

use remus::Device;

use super::{Lcdc, Ppu};

/// Pixel color values.
#[derive(Copy, Clone, Debug, Default)]
pub enum Color {
    /// Lightest
    #[default]
    C0 = 0b00,
    /// Light
    C1 = 0b01,
    /// Dark
    C2 = 0b10,
    /// Darkest
    C3 = 0b11,
}

#[derive(Debug)]
pub struct Pixel {
    pub color: Color,
    pub palette: Palette,
}

#[derive(Copy, Clone, Debug)]
pub enum Palette {
    BgWin,
    Obj0,
    Obj1,
}

#[derive(Debug, Default)]
pub struct Fifo(Vec<Pixel>);

impl Deref for Fifo {
    type Target = Vec<Pixel>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Fifo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Default)]
pub struct Fetch {
    stage: Stage,
    xpos: u8,
}

impl Fetch {
    pub fn exec(&mut self, fifo: &mut Fifo, ppu: &mut Ppu) {
        self.stage = std::mem::take(&mut self.stage).exec(self, fifo, ppu);
    }
}

#[derive(Debug, Default)]
pub enum Stage {
    #[default]
    ReadTile,
    ReadData0 {
        tile: u16,
    },
    ReadData1 {
        tile: u16,
        data0: u8,
    },
    Push([Pixel; 8]),
}

impl Stage {
    fn exec(self, fetch: &mut Fetch, fifo: &mut Fifo, ppu: &mut Ppu) -> Self {
        match self {
            Stage::ReadTile => {
                // Extract scanline config
                let regs = ppu.ctl.borrow();
                let lcdc = **regs.lcdc.borrow();
                let scy = **regs.scy.borrow();
                let scx = **regs.scx.borrow();
                let ly = **regs.ly.borrow();
                let wy = **regs.wy.borrow();
                let wx = **regs.wx.borrow();

                // Check if we should be drawing a window
                let win = Lcdc::WinEnable.get(&lcdc) && !(wy > ly || wx > 8 * fetch.xpos);

                // Calculate index of the tile
                let idx = {
                    let base = if win {
                        // Window tile
                        let winmap = Lcdc::WinMap.get(&lcdc);
                        [0x1800, 0x1c00][winmap as usize]
                    } else {
                        // Background tile
                        let bgmap = Lcdc::BgMap.get(&lcdc);
                        [0x1800, 0x1c00][bgmap as usize]
                    };
                    let ypos = (scy.wrapping_add(ly) / 8) as u16;
                    let xpos = ((scx / 8).wrapping_add(fetch.xpos) & 0x1f) as u16;
                    base + (32 * ypos) + xpos
                };

                // Increment x-position to next tile
                fetch.xpos += 1;

                // Fetch the tile data index
                let tile = ppu.vram.borrow().read(idx as usize);

                // Calculate the y-index of row within the tile
                let yoff = scy.wrapping_add(ly) % 8;
                let tile = if Lcdc::BgWinData.get(&lcdc) {
                    let base = 0x0000;
                    let tile = tile as u16;
                    let offset = (16 * tile) + (2 * yoff) as u16;
                    base + offset
                } else {
                    let base = 0x1000;
                    let tile = tile as i8 as i16;
                    let offset = (16 * tile) as i16 + (2 * yoff) as i16;
                    (base + offset) as u16
                };

                // Progress to next stage
                Stage::ReadData0 { tile }
            }
            Stage::ReadData0 { tile } => {
                // Fetch the first byte of the tile
                let data0 = ppu.vram.borrow().read(tile as usize);

                // Progress to next stage
                Stage::ReadData1 { tile, data0 }
            }
            Stage::ReadData1 { tile, data0 } => {
                // Fetch the seocnd byte of the tile
                let data1 = ppu.vram.borrow().read(tile as usize + 1);

                // Decode pixels from data
                let row = TileRow::from([data0, data1]).0;

                // Progress to next stage
                Stage::Push(row)
            }
            Stage::Push(row) => {
                // Push row to FIFO if there's space
                if fifo.len() <= 8 {
                    fifo.extend(row);
                    // Restart fetch
                    Stage::ReadTile
                } else {
                    // Try again next cycle
                    Stage::Push(row)
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct TileRow([Pixel; 8]);

impl From<[u8; 2]> for TileRow {
    fn from(bytes: [u8; 2]) -> Self {
        Self(
            (0..u8::BITS)
                .map(|bit| Pixel {
                    color: match (((bytes[0] & (0b1 << bit) != 0) as u8) << 1)
                        | ((bytes[1] & (0b1 << bit) != 0) as u8)
                    {
                        0b00 => Color::C0,
                        0b01 => Color::C1,
                        0b10 => Color::C2,
                        0b11 => Color::C3,
                        _ => unreachable!(),
                    },
                    palette: Palette::BgWin,
                })
                .collect::<Vec<Pixel>>()
                .try_into()
                .unwrap(),
        )
    }
}
