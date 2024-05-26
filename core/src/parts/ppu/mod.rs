//! Graphics model.

use rugby_arch::mem::Ram;
use rugby_arch::mio::{Bus, Mmio};
use rugby_arch::reg::{Port, Register};
use rugby_arch::{Block, Byte, Shared, Word};

use self::exec::hblank::HBlank;
use self::exec::vblank::VBlank;
use self::meta::pixel::{Palette, Pixel};
use super::dma::Control as Dma;
use super::pic::{self, Interrupt};
use crate::api::video::{self, Aspect, Video as Api};
use crate::dmg::pcb::Sram;

mod blk;
mod exec;
mod meta;

#[cfg(feature = "debug")]
pub mod dbg;

pub use self::exec::Mode;
pub use self::meta::pixel::Color;

/// Frame rate.
///
/// Video frame refresh occurs every 70,224 clock cycles. When running at the
/// full 4 MiHz, this equates to a frequency of ~59.7275 Hz.
#[allow(clippy::doc_markdown)]
pub const RATE: u32 = 70224;

/// Display resolution.
pub const LCD: Aspect = Aspect { wd: 160, ht: 144 };

/// Display framebuffer.
pub type Frame = video::Frame<Color, { LCD.depth() }>;

/// Video RAM.
///
/// 8 KiB RAM used to store tile [data][tdata] and [maps][tmaps].
///
/// [tdata]: https://gbdev.io/pandocs/Tile_Data.html
/// [tmaps]: https://gbdev.io/pandocs/Tile_Maps.html
pub type Vram = Sram;

/// Object attribute memory.
///
/// 160 byte RAM used to store sprites data. See more details [here][oam].
///
/// [oam]: https://gbdev.io/pandocs/OAM.html
pub type Oam = Ram<[Byte; 0x00a0]>;

/// Graphics register select.
#[derive(Clone, Copy, Debug)]
pub enum Select {
    /// `[$FF40]`: LCD control ([LCDC]).
    ///
    /// Main LCD control register, with bits controlling elements on the screen.
    ///
    /// | Bit | Name                        |
    /// |-----|-----------------------------|
    /// |  7  | LCD & PPU enable            |
    /// |  6  | Window tile map area        |
    /// |  5  | Window enable               |
    /// |  4  | BG & Window tile data area  |
    /// |  3  | BG tile map area            |
    /// |  2  | OBJ size                    |
    /// |  1  | OBJ enable                  |
    /// |  0  | BG & Window enable/priority |
    ///
    /// [lcdc]: https://gbdev.io/pandocs/LCDC.html
    Lcdc,
    /// `[$FF41]`: LCD status ([STAT]).
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
    /// `[$FF42]`: Viewport Y position.
    Scy,
    /// `[$FF43]`: Viewport X position.
    Scx,
    /// `[$FF44]`: LCD Y coordinate (read-only).
    Ly,
    /// `[$FF45]`: LY compare.
    ///
    /// Value to compare LY to for the LYC = LY interrupt source.
    Lyc,
    /// `[$FF46]`: OAM DMA source address.
    ///
    /// Writing to this register starts a [DMA] transfer from ROM or RAM to the
    /// [OAM](Oam).
    ///
    /// [dma]: https://gbdev.io/pandocs/OAM_DMA_Transfer.html
    Dma,
    /// `[$FF47]`: BG palette data.
    ///
    /// See more about palettes [here][palette].
    ///
    /// [palette]: https://gbdev.io/pandocs/Palettes.html
    Bgp,
    /// `[$FF48]`: OBJ palette 0 data
    ///
    /// See more about palettes [here][palette].
    ///
    /// [palette]: https://gbdev.io/pandocs/Palettes.html#ff48ff49--obp0-obp1-non-cgb-mode-only-obj-palette-0-1-data
    Obp0,
    /// `[$FF49]`: OBJ palette 1 data
    ///
    /// See more about palettes [here][palette].
    ///
    /// [palette]: https://gbdev.io/pandocs/Palettes.html#ff48ff49--obp0-obp1-non-cgb-mode-only-obj-palette-0-1-data
    Obp1,
    /// `[$FF4A]`: Window Y position.
    Wy,
    /// `[$FF4B]`: Window X position.
    Wx,
}

/// Picture processing unit.
#[derive(Debug)]
pub struct Ppu {
    /// Graphics registers.
    pub reg: Control,
    /// Graphics memory.
    pub mem: Bank,
    /// Graphics internals.
    etc: Internal,
    /// Interrupt line.
    int: pic::Line,
}

/// Graphics internals.
#[derive(Debug)]
struct Internal {
    /// Framebuffer.
    buf: Frame,
    /// Cycle count.
    dot: u16,
    /// Window line.
    win: Byte,
    /// Graphics mode.
    mode: Mode,
}

impl Internal {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Default for Internal {
    fn default() -> Self {
        Self {
            buf: [Color::default(); LCD.depth()],
            dot: Word::default(),
            win: Byte::default(),
            mode: Mode::default(),
        }
    }
}

impl Ppu {
    /// Constructs a new `Ppu`.
    #[must_use]
    pub fn new(vram: Shared<Vram>, oam: Shared<Oam>, dma: Shared<Dma>, int: pic::Line) -> Self {
        Self {
            reg: Control::new(dma),
            mem: Bank { vram, oam },
            etc: Internal::default(),
            int,
        }
    }

    /// Gets the current execution cycle.
    #[must_use]
    pub fn dot(&self) -> u16 {
        self.etc.dot
    }

    /// Gets the current execution mode.
    #[must_use]
    pub fn mode(&self) -> &Mode {
        &self.etc.mode
    }

    /// Get a reference to the PPU's screen.
    #[must_use]
    pub fn screen(&self) -> &Frame {
        &self.etc.buf
    }

    /// Color a pixel using the current palette.
    fn color(&self, pixel: &Pixel) -> Color {
        let pal = match pixel.meta.pal {
            Palette::BgWin => self.reg.bgp.load(),
            Palette::Obp0 => self.reg.obp0.load(),
            Palette::Obp1 => self.reg.obp1.load(),
        };
        pixel.col.recolor(pal)
    }
}

impl Api for Ppu {
    const SIZE: Aspect = LCD;

    type Pixel = Color;

    fn vsync(&self) -> bool {
        // In order to consider the frame ready to be rendered, the following
        // conditions must be met:
        //
        // 1. PPU is enabled
        let enable = Block::ready(self);
        // 2. Mode is vertical blank
        let vblank = matches!(self.etc.mode, Mode::VBlank(_));
        // 3. Scanline is last of virtual frame
        let bottom = (VBlank::LAST - 1) == self.reg.ly.load().into();
        // 4. Dot is final of virtual frame
        let finish = (HBlank::DOTS - 1) == self.etc.dot;
        //
        // In brief, this will cause
        //
        // Return if all conditions are met
        enable && vblank && bottom && finish
    }

    fn frame(&self) -> &[Self::Pixel] {
        &self.etc.buf
    }
}

impl Block for Ppu {
    fn ready(&self) -> bool {
        Lcdc::Enable.get(&self.reg.lcdc.load())
    }

    fn cycle(&mut self) {
        self.etc.mode = std::mem::take(&mut self.etc.mode).exec(self);
    }

    fn reset(&mut self) {
        self.reg.reset();
        self.etc.reset();
    }
}

impl Mmio for Ppu {
    fn attach(&self, bus: &mut Bus) {
        self.reg.attach(bus);
    }
}

#[rustfmt::skip]
impl Port<Byte> for Ppu {
    type Select = Select;

    fn load(&self, reg: Self::Select) -> Byte {
        match reg {
            Select::Lcdc => self.reg.lcdc.load(),
            Select::Stat => self.reg.stat.load(),
            Select::Scy  => self.reg.scy.load(),
            Select::Scx  => self.reg.scx.load(),
            Select::Ly   => self.reg.ly.load(),
            Select::Lyc  => self.reg.lyc.load(),
            Select::Dma  => self.reg.dma.load(),
            Select::Bgp  => self.reg.bgp.load(),
            Select::Obp0 => self.reg.obp0.load(),
            Select::Obp1 => self.reg.obp1.load(),
            Select::Wy   => self.reg.wy.load(),
            Select::Wx   => self.reg.wx.load(),
        }
    }

    fn store(&mut self, reg: Self::Select, value: Byte) {
        match reg {
            Select::Lcdc => self.reg.lcdc.store(value),
            Select::Stat => self.reg.stat.store(value),
            Select::Scy  => self.reg.scy.store(value),
            Select::Scx  => self.reg.scx.store(value),
            Select::Ly   => self.reg.ly.store(value),
            Select::Lyc  => self.reg.lyc.store(value),
            Select::Dma  => self.reg.dma.store(value),
            Select::Bgp  => self.reg.bgp.store(value),
            Select::Obp0 => self.reg.obp0.store(value),
            Select::Obp1 => self.reg.obp1.store(value),
            Select::Wy   => self.reg.wy.store(value),
            Select::Wx   => self.reg.wx.store(value),
        }
    }
}

/// Graphics registers.
///
/// | Address | Size | Name | Description                   |
/// |:-------:|------|------|-------------------------------|
/// | `$FF40` | Byte | LCDC | LCD control                   |
/// | `$FF41` | Byte | STAT | LCD status                    |
/// | `$FF42` | Byte | SCY  | Viewport Y position           |
/// | `$FF43` | Byte | SCX  | Viewport X position           |
/// | `$FF44` | Byte | LY   | LCD Y coordinate              |
/// | `$FF45` | Byte | LYC  | LY compare                    |
/// | `$FF46` | Byte | DMA  | OAM DMA source address        |
/// | `$FF47` | Byte | BGP  | BG palette data               |
/// | `$FF48` | Byte | OBP0 | OBJ palette 0 data            |
/// | `$FF49` | Byte | OBP1 | OBJ palette 1 data            |
/// | `$FF4A` | Byte | WY   | Window Y position             |
/// | `$FF4B` | Byte | WX   | Window X position             |
#[rustfmt::skip]
#[derive(Debug)]
pub struct Control {
    /// LCD control
    pub lcdc: Shared<Byte>,
    /// LCD status
    pub stat: Shared<Byte>,
    /// Viewport Y position
    pub scy:  Shared<Byte>,
    /// Viewport X position
    pub scx:  Shared<Byte>,
    /// LCD Y coordinate
    pub ly:   Shared<Byte>,
    /// LY compare
    pub lyc:  Shared<Byte>,
    /// OAM DMA source address
    pub dma:  Shared<Dma>,
    /// BG palette data
    pub bgp:  Shared<Byte>,
    /// OBJ palette 0 data
    pub obp0: Shared<Byte>,
    /// OBJ palette 1 data
    pub obp1: Shared<Byte>,
    /// Window Y position
    pub wy:   Shared<Byte>,
    /// Window X position
    pub wx:   Shared<Byte>,
}

impl Control {
    /// Constructs a new `File`.
    #[rustfmt::skip]
    #[must_use]    pub fn new(dma: Shared<Dma>) -> Self {
        Self {
            lcdc: Shared::default(),
            stat: Shared::default(),
            scy:  Shared::default(),
            scx:  Shared::default(),
            ly:   Shared::default(),
            lyc:  Shared::default(),
            dma,
            bgp:  Shared::default(),
            obp0: Shared::default(),
            obp1: Shared::default(),
            wy:   Shared::default(),
            wx:   Shared::default(),
        }
    }
}

impl Block for Control {
    fn reset(&mut self) {}
}

impl Mmio for Control {
    fn attach(&self, bus: &mut Bus) {
        bus.map(0xff40..=0xff40, self.lcdc.clone().into());
        bus.map(0xff41..=0xff41, self.stat.clone().into());
        bus.map(0xff42..=0xff42, self.scy.clone().into());
        bus.map(0xff43..=0xff43, self.scx.clone().into());
        bus.map(0xff44..=0xff44, self.ly.clone().into());
        bus.map(0xff45..=0xff45, self.lyc.clone().into());
        bus.map(0xff46..=0xff46, self.dma.clone().into());
        bus.map(0xff47..=0xff47, self.bgp.clone().into());
        bus.map(0xff48..=0xff48, self.obp0.clone().into());
        bus.map(0xff49..=0xff49, self.obp1.clone().into());
        bus.map(0xff4a..=0xff4a, self.wy.clone().into());
        bus.map(0xff4b..=0xff4b, self.wx.clone().into());
    }
}

/// Graphics memory.
///
/// |     Address     |  Size  | Name | Description   |
/// |:---------------:|--------|------|---------------|
/// | `$8000..=$9FFF` |  8 KiB | VRAM | Video RAM     |
/// | `$FE00..=$FEA0` |  160 B | OAM  | Object memory |
#[derive(Debug)]
pub struct Bank {
    /// Video RAM.
    pub vram: Shared<Vram>,
    /// Object memory.
    pub oam: Shared<Oam>,
}

/// Graphics control register.
#[rustfmt::skip]
#[derive(Clone, Copy, Debug)]
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
    /// Gets the value of the corresponding bit to the flag.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[must_use]
    fn get(self, value: &Byte) -> bool {
        *value & self as Byte != 0
    }
}
