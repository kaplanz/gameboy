//! Debugging the [PPU](super).

use itertools::Itertools;
use remus::mem::Memory;
use remus::reg::Register;
use remus::Word;

use super::blk::fetch::{Fetch, Location};
use super::meta::pixel::{Meta, Palette, Pixel};
use super::meta::tile::Tile;
use super::{Color, Lcdc, Ppu};

/// Collects debug information.
#[must_use]
pub fn info(ppu: &Ppu) -> Debug {
    Debug::new(ppu)
}

/// Debug information.
#[derive(Debug)]
pub struct Debug {
    /// Tile data.
    ///
    /// Rendering of the VRAM's tile data (`$8000..=$97FF`).
    pub tdat: Box<[Color; 0x06000]>,
    /// Tile map 1.
    ///
    /// Rendering of the VRAM's tile map 1 (`$9800..=$9BFF`).
    pub map1: Box<[Color; 0x10000]>,
    /// Tile map 2.
    ///
    /// Rendering of the VRAM's tile map 1 (`$9C00..=$9FFF`).
    pub map2: Box<[Color; 0x10000]>,
}

impl Debug {
    /// Constructs a new `Debug`.
    fn new(ppu: &Ppu) -> Self {
        // Extract scanline info
        let loc = Location::default();
        let bgw = Lcdc::BgMap.get(ppu.reg.lcdc.load());
        let tidx = |tnum| usize::from(Fetch::tidx(loc, bgw, tnum) >> 4);

        // Extract tile data, maps
        let tdat: [_; 0x180] = (0..0x1800)
            .map(|addr: Word| ppu.mem.vram.read(addr).unwrap())
            .collect_vec()
            .chunks_exact(16) // 16-bytes per tile
            .map(|tile| Tile::from(<[_; 16]>::try_from(tile).unwrap()))
            .collect_vec()
            .try_into()
            .unwrap();
        let map1: [_; 0x400] = (0x1800..0x1c00)
            .map(|addr: Word| ppu.mem.vram.read(addr).unwrap())
            .map(|tnum| tdat[tidx(tnum)].clone())
            .collect_vec()
            .try_into()
            .unwrap();
        let map2: [_; 0x400] = (0x1c00..0x2000)
            .map(|addr: Word| ppu.mem.vram.read(addr).unwrap())
            .map(|tnum| tdat[tidx(tnum)].clone())
            .collect_vec()
            .try_into()
            .unwrap();

        // Render tile data, maps
        let meta = Meta {
            pal: Palette::BgWin,
            bgp: false,
        }; // prepare metadata
        let tdat = Self::render(&tdat, ppu, meta, 16); // 16x24 tiles
        let map1 = Self::render(&map1, ppu, meta, 32); // 32x32 tiles
        let map2 = Self::render(&map2, ppu, meta, 32); // 32x32 tiles

        // Return debug info
        Self { tdat, map1, map2 }
    }

    /// Renders tile data as pixels.
    #[allow(clippy::unnecessary_box_returns)]
    fn render<const N: usize>(
        tdat: &[Tile],
        ppu: &Ppu,
        meta: Meta,
        width: usize,
    ) -> Box<[Color; N]> {
        tdat.chunks_exact(width) // tiles per row
            .flat_map(|row| {
                row.iter()
                    .flat_map(|tile| tile.iter().enumerate())
                    .sorted_by_key(|row| row.0)
                    .map(|(_, row)| row)
                    .collect_vec()
            })
            .flat_map(|row| row.into_iter().map(|col| ppu.color(&Pixel::new(col, meta))))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}
