//! Picture processing unit.

use itertools::Itertools;
use remus::bus::Bus;
use remus::dev::Device;
use remus::mem::Ram;
use remus::reg::Register;
use remus::{Address, Block, Board, Cell, Linked, Location, Machine, Shared};

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

/// Video RAM.
///
/// 8 KiB of RAM used to store tile [data][tdata] and [maps][tmaps].
///
/// [tdata]: https://gbdev.io/pandocs/Tile_Data.html
/// [tmaps]: https://gbdev.io/pandocs/Tile_Maps.html
pub type Vram = Ram<u8, 0x2000>;
/// Object Attribute Memory ([OAM][oam]).
///
/// 160 B of RAM used to store up to 40 sprites.
///
/// [oam]: https://gbdev.io/pandocs/OAM.html
pub type Oam = Ram<u8, 0x00a0>;

/// 8-bit LCD control register set.
#[derive(Clone, Copy, Debug)]
pub enum Control {
    /// `0xFF40`: LCD  Control ([LCDC][lcdc]).
    ///
    /// Main LCD control register, with bits controlling elements on the screen.
    ///
    /// | Bit | Name                           |
    /// |-----|--------------------------------|
    /// |  7  | LCD and PPU enable             |
    /// |  6  | Window tile map area           |
    /// |  5  | Window enable                  |
    /// |  4  | BG and Window tile data area   |
    /// |  3  | BG tile map area               |
    /// |  2  | OBJ size                       |
    /// |  1  | OBJ enable                     |
    /// |  0  | BG and Window enable/priority  |
    ///
    /// [lcdc]: https://gbdev.io/pandocs/LCDC.html
    Lcdc,
    /// `0xFF41`: LCD status ([STAT][stat]).
    ///
    /// Current LCD status register, also used to control PPU interrupts.
    ///
    /// | Bit | Name                           | Use |
    /// |-----|--------------------------------|-----|
    /// |  6  | STAT interrupt source (LYC=LY) | R/W |
    /// |  5  | OAM interrupt source           | R/W |
    /// |  4  | VBlank interrupt source        | R/W |
    /// |  3  | HBlank interrupt source        | R/W |
    /// |  2  | LYC = LY compare flag          | R   |
    /// | 1-0 | Mode flag (2-bit)              | R   |
    ///
    /// [stat]: https://gbdev.io/pandocs/STAT.html#ff41--stat-lcd-status
    Stat,
    /// `0xFF42`: Viewport Y position.
    Scy,
    /// `0xFF43`: Viewport X position.
    Scx,
    /// `0xFF44`: LCD Y coordinate (read-only).
    Ly,
    /// `0xFF45`: LY compare.
    ///
    /// Value to compare LY to for the LYC = LY interrupt source.
    Lyc,
    /// `0xFF46`: OAM DMA source address.
    ///
    /// Writing to this register starts a [DMA][dma] transfer from ROM or RAM to
    /// the [OAM](Oam).
    ///
    /// [dma]: https://gbdev.io/pandocs/OAM_DMA_Transfer.html
    Dma,
    /// `0xFF47`: BG palette data.
    ///
    /// See more about palettes [here][palette].
    ///
    /// [palette]: https://gbdev.io/pandocs/Palettes.html
    Bgp,
    /// `0xFF48`: OBJ palette 0 data
    Obp0,
    /// `0xFF48`: OBJ palette 1 data
    Obp1,
    /// `0xFF4A`: Window Y position.
    Wy,
    /// `0xFF4B`: Window X position.
    Wx,
}

/// PPU model.
#[derive(Debug, Default)]
pub struct Ppu {
    // State
    dot: usize,
    winln: u8,
    mode: Mode,
    // Output
    lcd: Screen,
    // Control
    // ┌──────┬──────────┬─────┐
    // │ Size │   Name   │ Dev │
    // ├──────┼──────────┼─────┤
    // │ 12 B │ Control  │ Reg │
    // └──────┴──────────┴─────┘
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
    // Shared
    bus: Shared<Bus<u16, u8>>,
    pic: Shared<Pic>,
}

impl Ppu {
    /// Constructs a new `Ppu`.
    #[must_use]
    pub fn new(bus: Shared<Bus<u16, u8>>, pic: Shared<Pic>) -> Self {
        // Construct shared blocks
        let oam = Shared::from(Oam::default());
        // Construct self
        Self {
            file: File::new(bus.clone(), oam.clone()),
            oam,
            bus,
            pic,
            ..Default::default()
        }
    }

    /// Gets internal debug info.
    #[must_use]
    pub fn debug(&self) -> Debug {
        Debug::new(self)
    }

    /// Get a reference to the PPU's screen.
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
        let topline = self.file.ly.load() == 0;
        // 3. Dot is first of scanline
        let firstdot = self.dot == 0;

        enabled && topline && firstdot
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

    /// Color a pixel according to the ppu's palette configuration.
    fn color(&self, pixel: &Pixel) -> Color {
        let pal = match pixel.meta.pal {
            Palette::BgWin => self.file.bgp.load(),
            Palette::Obp0 => self.file.obp0.load(),
            Palette::Obp1 => self.file.obp1.load(),
        };
        pixel.col.recolor(pal)
    }
}

impl Block for Ppu {
    fn reset(&mut self) {
        // State
        std::mem::take(&mut self.dot);
        std::mem::take(&mut self.winln);
        std::mem::take(&mut self.mode);
        // Control
        self.file.reset();
        // Memory
        self.vram.reset();
        self.oam.reset();
    }
}

impl Board<u16, u8> for Ppu {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus<u16, u8>) {
        // Connect boards
        self.file.connect(bus);

        // Extract devices
        let vram = self.vram().to_dynamic();
        let oam  = self.oam().to_dynamic();

        // Map devices on bus           // ┌──────┬────────┬────────┬─────┐
                                        // │ Addr │  Size  │  Name  │ Dev │
                                        // ├──────┼────────┼────────┼─────┤
        bus.map(0x8000..=0x9fff, vram); // │ 8000 │  8 KiB │ Video  │ RAM │
        bus.map(0xfe00..=0xfe9f, oam);  // │ fe00 │  160 B │ Object │ RAM │
                                        // └──────┴────────┴────────┴─────┘
    }
}

impl Linked<Bus<u16, u8>> for Ppu {
    fn mine(&self) -> Shared<Bus<u16, u8>> {
        self.bus.clone()
    }

    fn link(&mut self, it: Shared<Bus<u16, u8>>) {
        self.bus = it;
    }
}

impl Linked<Pic> for Ppu {
    fn mine(&self) -> Shared<Pic> {
        self.pic.clone()
    }

    fn link(&mut self, it: Shared<Pic>) {
        self.pic = it;
    }
}

#[rustfmt::skip]
impl Location<u8> for Ppu {
    type Register = Control;

    fn load(&self, reg: Self::Register) -> u8 {
        match reg {
            Control::Lcdc => self.file.lcdc.load(),
            Control::Stat => self.file.stat.load(),
            Control::Scy  => self.file.scy.load(),
            Control::Scx  => self.file.scx.load(),
            Control::Ly   => self.file.ly.load(),
            Control::Lyc  => self.file.lyc.load(),
            Control::Dma  => self.file.dma.load(),
            Control::Bgp  => self.file.bgp.load(),
            Control::Obp0 => self.file.obp0.load(),
            Control::Obp1 => self.file.obp1.load(),
            Control::Wy   => self.file.wy.load(),
            Control::Wx   => self.file.wx.load(),
        }
    }

    fn store(&mut self, reg: Self::Register, value: u8) {
        match reg {
            Control::Lcdc => self.file.lcdc.store(value),
            Control::Stat => self.file.stat.store(value),
            Control::Scy  => self.file.scy.store(value),
            Control::Scx  => self.file.scx.store(value),
            Control::Ly   => self.file.ly.store(value),
            Control::Lyc  => self.file.lyc.store(value),
            Control::Dma  => self.file.dma.store(value),
            Control::Bgp  => self.file.bgp.store(value),
            Control::Obp0 => self.file.obp0.store(value),
            Control::Obp1 => self.file.obp1.store(value),
            Control::Wy   => self.file.wy.store(value),
            Control::Wx   => self.file.wx.store(value),
        }
    }
}

impl Machine for Ppu {
    fn enabled(&self) -> bool {
        Lcdc::Enable.get(self.file.lcdc.load())
    }

    fn cycle(&mut self) {
        self.mode = std::mem::take(&mut self.mode).exec(self);
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

impl File {
    /// Constructs a new `File`.
    pub fn new(bus: Shared<Bus<u16, u8>>, oam: Shared<Oam>) -> Self {
        Self {
            dma: Dma::new(bus, oam).into(),
            ..Default::default()
        }
    }
}

impl Block for File {
    fn reset(&mut self) {}
}

impl Board<u16, u8> for File {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus<u16, u8>) {
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

        // Map devices on bus           // ┌──────┬──────┬────────────────┬─────┐
                                        // │ Addr │ Size │      Name      │ Dev │
                                        // ├──────┼──────┼────────────────┼─────┤
        bus.map(0xff40..=0xff40, lcdc); // │ ff40 │  1 B │ LCD Control    │ Reg │
        bus.map(0xff41..=0xff41, stat); // │ ff41 │  1 B │ LCD Status     │ Reg │
        bus.map(0xff42..=0xff42, scy);  // │ ff42 │  1 B │ Scroll Y       │ Reg │
        bus.map(0xff43..=0xff43, scx);  // │ ff43 │  1 B │ Scroll X       │ Reg │
        bus.map(0xff44..=0xff44, ly);   // │ ff44 │  1 B │ LCD Y          │ Reg │
        bus.map(0xff45..=0xff45, lyc);  // │ ff45 │  1 B │ LY Compare     │ Reg │
        bus.map(0xff46..=0xff46, dma);  // │ ff46 │  1 B │ DMA Start      │ DMA │
        bus.map(0xff47..=0xff47, bgp);  // │ ff47 │  1 B │ BG Palette     │ Reg │
        bus.map(0xff48..=0xff48, obp0); // │ ff48 │  1 B │ OBJ Palette 0  │ Reg │
        bus.map(0xff49..=0xff49, obp1); // │ ff49 │  1 B │ OBJ Palette 1  │ Reg │
        bus.map(0xff4a..=0xff4a, wy);   // │ ff4a │  1 B │ Window Y       │ Reg │
        bus.map(0xff4b..=0xff4b, wx);   // │ ff4b │  1 B │ Window X       │ Reg │
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

/// Debug information.
#[derive(Debug)]
pub struct Debug {
    /// Tile data.
    pub tdat: [Color; 0x06000],
    /// Tile map 1.
    pub map1: [Color; 0x10000],
    /// Tile map 2.
    pub map2: [Color; 0x10000],
}

impl Debug {
    /// Constructs `Debug` info for the PPU.
    fn new(ppu: &Ppu) -> Self {
        // Extract scanline info
        let lcdc = ppu.file.lcdc.load();
        let bgwin = Lcdc::BgMap.get(lcdc);

        // Retrieve a copy of the VRAM
        let vram = ppu.vram.borrow();

        // Extract tile data, maps
        let tdat: [_; 0x180] = (0..0x1800)
            .map(|addr: u16| vram.read(addr))
            .collect_vec()
            .chunks_exact(16) // 16-bytes per tile
            .map(|tile| Tile::from(<[_; 16]>::try_from(tile).unwrap()))
            .collect_vec()
            .try_into()
            .unwrap();
        let map1: [_; 0x400] = (0x1800..0x1c00)
            .map(|addr: u16| vram.read(addr))
            .map(|tnum| tdat[Self::tidx(tnum, bgwin)].clone())
            .collect_vec()
            .try_into()
            .unwrap();
        let map2: [_; 0x400] = (0x1c00..0x2000)
            .map(|addr: u16| vram.read(addr))
            .map(|tnum| tdat[Self::tidx(tnum, bgwin)].clone())
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

    /// Fetches the appropriate tile address from an tile number
    #[allow(clippy::identity_op)]
    fn tidx(tnum: u8, bgwin: bool) -> usize {
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
