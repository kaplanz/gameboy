//! Picture processing unit.

use std::cell::RefCell;
use std::rc::Rc;

use itertools::Itertools;
use remus::bus::Bus;
use remus::mem::Ram;
use remus::reg::Register;
use remus::{Address, Block, Board, Machine, Processor, Shared};

use self::dma::Dma;
use self::exec::Mode;
use self::pixel::{Meta, Palette, Pixel};
use self::tile::Tile;
use super::pic::{Interrupt, Pic};
use crate::dmg::SCREEN;

mod blk;
mod dma;
mod exec;
mod pixel;
mod screen;
mod sprite;
mod tile;

pub use self::pixel::Color;
pub use self::screen::Screen;

pub type Vram = Ram<0x2000>;
pub type Oam = Ram<0x00a0>;

/// 8-bit register set.
#[derive(Clone, Copy, Debug)]
pub enum Control {
    Lcdc,
    Stat,
    Scy,
    Scx,
    Ly,
    Lyc,
    Dma,
    Bgp,
    Obp0,
    Obp1,
    Wy,
    Wx,
}

/// PPU model.
#[derive(Debug, Default)]
pub struct Ppu {
    // State
    dot: usize,
    winln: u8,
    mode: Mode,
    // Connections
    bus: Shared<Bus>,
    pic: Rc<RefCell<Pic>>,
    // Control
    // ┌────────┬──────────┬─────┬───────┐
    // │  Size  │   Name   │ Dev │ Alias │
    // ├────────┼──────────┼─────┼───────┤
    // │   12 B │ Control  │ Reg │       │
    // └────────┴──────────┴─────┴───────┘
    file: File,
    // Memory
    // ┌────────┬──────────┬─────┬───────┐
    // │  Size  │   Name   │ Dev │ Alias │
    // ├────────┼──────────┼─────┼───────┤
    // │  8 KiB │ Video    │ RAM │ VRAM  │
    // │  160 B │ Object   │ RAM │ OAM   │
    // └────────┴──────────┴─────┴───────┘
    vram: Shared<Vram>,
    oam: Shared<Oam>,
    // Outputs
    lcd: Screen,
}

impl Ppu {
    /// Gets internal debug info.
    #[must_use]
    pub fn debug(&self) -> Debug {
        Debug::new(self)
    }

    /// Gets a shared reference to the PPU's video RAM.
    #[must_use]
    pub fn vram(&self) -> Shared<Vram> {
        self.vram.clone()
    }

    /// Gets a shared reference to the PPU's object attribute RAM.
    #[must_use]
    pub fn oam(&self) -> Shared<Oam> {
        self.oam.clone()
    }

    /// Gets a shared reference to the PPU's LCD control register.
    #[must_use]
    pub fn lcdc(&self) -> Shared<Register<u8>> {
        self.file.lcdc.clone()
    }

    /// Gets a shared reference to the PPU's LCD status register.
    #[must_use]
    pub fn stat(&self) -> Shared<Register<u8>> {
        self.file.stat.clone()
    }

    /// Gets a shared reference to the PPU's scroll Y register.
    #[must_use]
    pub fn scy(&self) -> Shared<Register<u8>> {
        self.file.scy.clone()
    }

    /// Gets a shared reference to the PPU's scroll X register.
    #[must_use]
    pub fn scx(&self) -> Shared<Register<u8>> {
        self.file.scx.clone()
    }

    /// Gets a shared reference to the PPU's LCD Y register.
    #[must_use]
    pub fn ly(&self) -> Shared<Register<u8>> {
        self.file.ly.clone()
    }

    /// Gets a shared reference to the PPU's LY compare register.
    #[must_use]
    pub fn lyc(&self) -> Shared<Register<u8>> {
        self.file.lyc.clone()
    }

    /// Gets a shared reference to the PPU's DMA start register.
    #[must_use]
    pub fn dma(&self) -> Shared<Dma> {
        self.file.dma.clone()
    }

    /// Gets a shared reference to the PPU's BG palette register.
    #[must_use]
    pub fn bgp(&self) -> Shared<Register<u8>> {
        self.file.bgp.clone()
    }

    /// Gets a shared reference to the PPU's OBJ palette 0 register.
    #[must_use]
    pub fn obp0(&self) -> Shared<Register<u8>> {
        self.file.obp0.clone()
    }

    /// Gets a shared reference to the PPU's OBJ palette 1 register.
    #[must_use]
    pub fn obp1(&self) -> Shared<Register<u8>> {
        self.file.obp1.clone()
    }

    /// Gets a shared reference to the PPU's window Y register.
    #[must_use]
    pub fn wy(&self) -> Shared<Register<u8>> {
        self.file.wy.clone()
    }

    /// Gets a shared reference to the PPU's window X register.
    #[must_use]
    pub fn wx(&self) -> Shared<Register<u8>> {
        self.file.wx.clone()
    }

    /// Set the ppu's bus.
    pub fn set_bus(&mut self, bus: Shared<Bus>) {
        self.bus = bus;
    }

    /// Set the ppu's pic.
    pub fn set_pic(&mut self, pic: Rc<RefCell<Pic>>) {
        self.pic = pic;
    }

    /// Get a reference to the ppu's screen.
    #[must_use]
    pub fn screen(&self) -> &Screen {
        &self.lcd
    }

    /// Check if the screen is ready to be redrawn.
    #[must_use]
    pub fn ready(&self) -> bool {
        // Redraw the screen once per frame, when:
        // 1. PPU is enabled
        let enabled = self.enabled();
        // 2. Scanline is top of screen
        let topline = **self.file.ly.borrow() == 0;
        // 3. Dot is first of scanline
        let firstdot = self.dot == 0;

        enabled && topline && firstdot
    }

    /// Color a pixel according to the ppu's palette configuration.
    fn color(&self, pixel: &Pixel) -> Color {
        let pal = **match pixel.meta.pal {
            Palette::BgWin => self.file.bgp.borrow(),
            Palette::Obp0 => self.file.obp0.borrow(),
            Palette::Obp1 => self.file.obp1.borrow(),
        };
        pixel.col.recolor(pal)
    }
}

impl Block for Ppu {
    fn reset(&mut self) {
        // Reset mode
        self.mode = Mode::default();

        // Reset control
        self.file.reset();

        // Reset DMA
        self.file.dma.borrow_mut().set_bus(self.bus.clone());
        self.file.dma.borrow_mut().set_oam(self.oam.clone());

        // Reset memory
        self.vram.borrow_mut().reset();
        self.oam.borrow_mut().reset();

        // Reset LCD
        self.lcd = Screen::default();
    }
}

impl Board for Ppu {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Connect boards
        self.file.connect(bus);

        // Extract memory
        let vram = self.vram().to_dynamic();
        let oam  = self.oam().to_dynamic();

        // Map devices on bus  // ┌──────┬────────┬────────┬─────┐
                               // │ Addr │  Size  │  Name  │ Dev │
                               // ├──────┼────────┼────────┼─────┤
        bus.map(0x8000, vram); // │ 8000 │  8 KiB │ Video  │ RAM │
        bus.map(0xfe00, oam);  // │ fe00 │  160 B │ Object │ RAM │
                               // └──────┴────────┴────────┴─────┘
    }
}

impl Machine for Ppu {
    fn enabled(&self) -> bool {
        Lcdc::Enable.get(**self.file.lcdc.borrow())
    }

    fn cycle(&mut self) {
        self.mode = std::mem::take(&mut self.mode).exec(self);

        // Cycle the DMA every machine cycle
        let mut dma = self.file.dma.borrow_mut();
        if dma.enabled() && self.dot % 4 == 0 {
            dma.cycle();
        }
    }
}

#[rustfmt::skip]
impl Processor<u8> for Ppu {
    type Register = Control;

    fn load(&self, reg: Self::Register) -> u8 {
        match reg {
            Control::Lcdc => **self.file.lcdc.borrow(),
            Control::Stat => **self.file.stat.borrow(),
            Control::Scy  => **self.file.scy.borrow(),
            Control::Scx  => **self.file.scx.borrow(),
            Control::Ly   => **self.file.ly.borrow(),
            Control::Lyc  => **self.file.lyc.borrow(),
            Control::Dma  =>   self.file.dma.read(0),
            Control::Bgp  => **self.file.bgp.borrow(),
            Control::Obp0 => **self.file.obp0.borrow(),
            Control::Obp1 => **self.file.obp1.borrow(),
            Control::Wy   => **self.file.wy.borrow(),
            Control::Wx   => **self.file.wx.borrow(),
        }
    }

    fn store(&mut self, reg: Self::Register, value: u8) {
        match reg {
            Control::Lcdc => **self.file.lcdc.borrow_mut() = value,
            Control::Stat => **self.file.stat.borrow_mut() = value,
            Control::Scy  => **self.file.scy.borrow_mut()  = value,
            Control::Scx  => **self.file.scx.borrow_mut()  = value,
            Control::Ly   => **self.file.ly.borrow_mut()   = value,
            Control::Lyc  => **self.file.lyc.borrow_mut()  = value,
            Control::Dma  =>   self.file.dma.write(0, value),
            Control::Bgp  => **self.file.bgp.borrow_mut()  = value,
            Control::Obp0 => **self.file.obp0.borrow_mut() = value,
            Control::Obp1 => **self.file.obp1.borrow_mut() = value,
            Control::Wy   => **self.file.wy.borrow_mut()   = value,
            Control::Wx   => **self.file.wx.borrow_mut()   = value,
        }
    }
}

/// PPU control register file.
#[rustfmt::skip]
#[derive(Debug, Default)]
struct File {
    // ┌──────┬────────────────┬─────┬───────┐
    // │ Size │      Name      │ Dev │ Alias │
    // ├──────┼────────────────┼─────┼───────┤
    // │  1 B │ LCD Control    │ Reg │ LCDC  │
    // │  1 B │ LCD Status     │ Reg │ STAT  │
    // │  1 B │ Scroll Y       │ Reg │ SCY   │
    // │  1 B │ Scroll X       │ Reg │ SCX   │
    // │  1 B │ LCD Y          │ Reg │ LY    │
    // │  1 B │ LY Compare     │ Reg │ LYC   │
    // │  1 B │ DMA Start      │ DMA │ DMA   │
    // │  1 B │ BG Palette     │ Reg │ BGP   │
    // │  1 B │ OBJ Palette 0  │ Reg │ OBP0  │
    // │  1 B │ OBJ Palette 1  │ Reg │ OBP1  │
    // │  1 B │ Window Y       │ Reg │ WY    │
    // │  1 B │ Window X       │ Reg │ WX    │
    // └──────┴────────────────┴─────┴───────┘
    lcdc: Shared<Register<u8>>,
    stat: Shared<Register<u8>>,
    scy:  Shared<Register<u8>>,
    scx:  Shared<Register<u8>>,
    ly:   Shared<Register<u8>>,
    lyc:  Shared<Register<u8>>,
    dma:  Shared<Dma>,
    bgp:  Shared<Register<u8>>,
    obp0: Shared<Register<u8>>,
    obp1: Shared<Register<u8>>,
    wy:   Shared<Register<u8>>,
    wx:   Shared<Register<u8>>,
}

impl Block for File {
    fn reset(&mut self) {}
}

impl Board for File {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let lcdc = self.lcdc.clone().to_dynamic();
        let stat = self.stat.clone().to_dynamic();
        let scy  = self.scy.clone().to_dynamic();
        let scx  = self.scx.clone().to_dynamic();
        let ly   = self.ly.clone().to_dynamic();
        let lyc  = self.lyc.clone().to_dynamic();
        let dma  = self.dma.clone().to_dynamic();
        let bgp  = self.bgp.clone().to_dynamic();
        let obp0 = self.obp0.clone().to_dynamic();
        let obp1 = self.obp1.clone().to_dynamic();
        let wy   = self.wy.clone().to_dynamic();
        let wx   = self.wx.clone().to_dynamic();

        // Map devices on bus   // ┌──────┬──────┬────────────────┬─────┐
                                // │ Addr │ Size │      Name      │ Dev │
                                // ├──────┼──────┼────────────────┼─────┤
        bus.map(0xff40, lcdc);  // │ ff40 │  1 B │ LCD Control    │ Reg │
        bus.map(0xff41, stat);  // │ ff41 │  1 B │ LCD Status     │ Reg │
        bus.map(0xff42, scy);   // │ ff42 │  1 B │ Scroll Y       │ Reg │
        bus.map(0xff43, scx);   // │ ff43 │  1 B │ Scroll X       │ Reg │
        bus.map(0xff44, ly);    // │ ff44 │  1 B │ LCD Y          │ Reg │
        bus.map(0xff45, lyc);   // │ ff45 │  1 B │ LY Compare     │ Reg │
        bus.map(0xff46, dma);   // │ ff46 │  1 B │ DMA Start      │ DMA │
        bus.map(0xff47, bgp);   // │ ff47 │  1 B │ BG Palette     │ Reg │
        bus.map(0xff48, obp0);  // │ ff48 │  1 B │ OBJ Palette 0  │ Reg │
        bus.map(0xff49, obp1);  // │ ff49 │  1 B │ OBJ Palette 1  │ Reg │
        bus.map(0xff4a, wy);    // │ ff4a │  1 B │ Window Y       │ Reg │
        bus.map(0xff4b, wx);    // │ ff4b │  1 B │ Window X       │ Reg │
                                // └──────┴──────┴────────────────┴─────┘
    }
}

#[rustfmt::skip]
#[derive(Copy, Clone, Debug)]
enum Lcdc {
    Enable      = 0b1000_0000,
    WinMap      = 0b0100_0000,
    WinEnable   = 0b0010_0000,
    BgWinData   = 0b0001_0000,
    BgMap       = 0b0000_1000,
    ObjSize     = 0b0000_0100,
    ObjEnable   = 0b0000_0010,
    BgWinEnable = 0b0000_0001,
}

impl Lcdc {
    pub fn get(self, lcdc: u8) -> bool {
        lcdc & self as u8 != 0
    }
}

#[derive(Debug)]
pub struct Debug {
    pub tdat: [Color; 0x06000],
    pub map1: [Color; 0x10000],
    pub map2: [Color; 0x10000],
}

impl Debug {
    /// Constructs `Debug` info for the PPU.
    fn new(ppu: &Ppu) -> Self {
        // Extract scanline info
        let lcdc = **ppu.file.lcdc.borrow();
        let bgwin = Lcdc::BgMap.get(lcdc);

        // Retrieve a copy of the VRAM
        let vram = ppu.vram.borrow();

        // Extract tile data, maps
        let tdat: [_; 0x180] = vram[..0x1800]
            .chunks_exact(16) // 16-bytes per tile
            .map(|tile| Tile::from(<[_; 16]>::try_from(tile).unwrap()))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let map1: [_; 0x400] = vram[0x1800..0x1c00]
            .iter()
            .map(|&tnum| tdat[Self::tidx(tnum, bgwin)].clone())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let map2: [_; 0x400] = vram[0x1c00..0x2000]
            .iter()
            .map(|&tnum| tdat[Self::tidx(tnum, bgwin)].clone())
            .collect::<Vec<_>>()
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

    /// Fetches the appropriate tile address from an tile number
    #[allow(clippy::identity_op)]
    pub fn tidx(tnum: u8, bgwin: bool) -> usize {
        // Calculate tile index offset
        let addr = if bgwin {
            (0x1000i16 + (16 * tnum as i8 as i16)) as usize
        } else {
            (0x0000u16 + (16 * tnum as u16)) as usize
        };
        addr / 16
    }

    /// Renders tile data as pixels
    fn render<const N: usize>(tdat: &[Tile], ppu: &Ppu, meta: Meta, width: usize) -> [Color; N] {
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
