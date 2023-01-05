//! Picture processing unit.

use std::cell::RefCell;
use std::rc::Rc;

use remus::bus::Bus;
use remus::mem::Ram;
use remus::reg::Register;
use remus::{Block, Machine, SharedDevice};

use self::dma::Dma;
use self::exec::Mode;
use self::pixel::{Color, Palette, Pixel};
use super::pic::{Interrupt, Pic};
use crate::dmg::{Board, SCREEN};

mod blk;
mod dma;
mod exec;
mod pixel;
mod screen;
mod sprite;

pub use self::screen::Screen;

pub type Vram = Ram<0x2000>;
pub type Oam = Ram<0x00a0>;

/// PPU model.
#[derive(Debug, Default)]
pub struct Ppu {
    // State
    dot: usize,
    winln: u8,
    mode: Mode,
    // Connections
    bus: Rc<RefCell<Bus>>,
    pic: Rc<RefCell<Pic>>,
    // Control
    // ┌────────┬──────────┬─────┬───────┐
    // │  Size  │   Name   │ Dev │ Alias │
    // ├────────┼──────────┼─────┼───────┤
    // │   12 B │ Control  │ Reg │       │
    // └────────┴──────────┴─────┴───────┘
    ctl: Control,
    // Devices
    // ┌────────┬──────────┬─────┬───────┐
    // │  Size  │   Name   │ Dev │ Alias │
    // ├────────┼──────────┼─────┼───────┤
    // │  8 KiB │ Video    │ RAM │ VRAM  │
    // │  160 B │ Object   │ RAM │ OAM   │
    // └────────┴──────────┴─────┴───────┘
    vram: Rc<RefCell<Vram>>,
    oam: Rc<RefCell<Oam>>,
    // Outputs
    lcd: Screen,
}

impl Ppu {
    /// Gets a shared reference to the PPU's video RAM.
    #[must_use]
    pub fn vram(&self) -> SharedDevice {
        self.vram.clone()
    }

    /// Gets a shared reference to the PPU's object attribute RAM.
    #[must_use]
    pub fn oam(&self) -> SharedDevice {
        self.oam.clone()
    }

    /// Gets a shared reference to the PPU's LCD control register.
    #[must_use]
    pub fn lcdc(&self) -> SharedDevice {
        self.ctl.lcdc.clone()
    }

    /// Gets a shared reference to the PPU's LCD status register.
    #[must_use]
    pub fn stat(&self) -> SharedDevice {
        self.ctl.stat.clone()
    }

    /// Gets a shared reference to the PPU's scroll Y register.
    #[must_use]
    pub fn scy(&self) -> SharedDevice {
        self.ctl.scy.clone()
    }

    /// Gets a shared reference to the PPU's scroll X register.
    #[must_use]
    pub fn scx(&self) -> SharedDevice {
        self.ctl.scx.clone()
    }

    /// Gets a shared reference to the PPU's LCD Y register.
    #[must_use]
    pub fn ly(&self) -> SharedDevice {
        self.ctl.ly.clone()
    }

    /// Gets a shared reference to the PPU's LY compare register.
    #[must_use]
    pub fn lyc(&self) -> SharedDevice {
        self.ctl.lyc.clone()
    }

    /// Gets a shared reference to the PPU's DMA start register.
    #[must_use]
    pub fn dma(&self) -> SharedDevice {
        self.ctl.dma.clone()
    }

    /// Gets a shared reference to the PPU's BG palette register.
    #[must_use]
    pub fn bgp(&self) -> SharedDevice {
        self.ctl.bgp.clone()
    }

    /// Gets a shared reference to the PPU's OBJ palette 0 register.
    #[must_use]
    pub fn obp0(&self) -> SharedDevice {
        self.ctl.obp0.clone()
    }

    /// Gets a shared reference to the PPU's OBJ palette 1 register.
    #[must_use]
    pub fn obp1(&self) -> SharedDevice {
        self.ctl.obp1.clone()
    }

    /// Gets a shared reference to the PPU's window Y register.
    #[must_use]
    pub fn wy(&self) -> SharedDevice {
        self.ctl.wy.clone()
    }

    /// Gets a shared reference to the PPU's window X register.
    #[must_use]
    pub fn wx(&self) -> SharedDevice {
        self.ctl.wx.clone()
    }

    /// Set the ppu's bus.
    pub fn set_bus(&mut self, bus: Rc<RefCell<Bus>>) {
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
        let topline = **self.ctl.ly.borrow() == 0;
        // 3. Dot is first of scanline
        let firstdot = self.dot == 0;

        enabled && topline && firstdot
    }

    /// Color a pixel according to the ppu's palette configuration.
    fn color(&self, pixel: &Pixel) -> Color {
        let pal = **match pixel.pal() {
            Palette::BgWin => self.ctl.bgp.borrow(),
            Palette::Obj0 => self.ctl.obp0.borrow(),
            Palette::Obj1 => self.ctl.obp1.borrow(),
        };
        pixel.col().recolor(pal)
    }
}

impl Block for Ppu {
    fn reset(&mut self) {
        // Reset LCD
        self.lcd = Screen::default();

        // Reset mode
        self.mode = Mode::default();

        // Reset memory
        self.vram.borrow_mut().reset();
        self.oam.borrow_mut().reset();

        // Reset registers
        self.ctl.reset();

        // Reset DMA
        self.ctl.dma.borrow_mut().set_bus(self.bus.clone());
        self.ctl.dma.borrow_mut().set_oam(self.oam.clone());
    }
}

impl Board for Ppu {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Connect boards
        self.ctl.connect(bus);

        // Extract devices
        let vram = self.vram();
        let oam  = self.oam();

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
        Lcdc::Enable.get(**self.ctl.lcdc.borrow())
    }

    fn cycle(&mut self) {
        self.mode = std::mem::take(&mut self.mode).exec(self);

        // Cycle the DMA every machine cycle
        let mut dma = self.ctl.dma.borrow_mut();
        if dma.enabled() && self.dot % 4 == 0 {
            dma.cycle();
        }
    }
}

/// Control registers.
#[rustfmt::skip]
#[derive(Debug, Default)]
pub struct Control {
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
    lcdc: Rc<RefCell<Register<u8>>>,
    stat: Rc<RefCell<Register<u8>>>,
    scy:  Rc<RefCell<Register<u8>>>,
    scx:  Rc<RefCell<Register<u8>>>,
    ly:   Rc<RefCell<Register<u8>>>,
    lyc:  Rc<RefCell<Register<u8>>>,
    dma:  Rc<RefCell<Dma>>,
    bgp:  Rc<RefCell<Register<u8>>>,
    obp0: Rc<RefCell<Register<u8>>>,
    obp1: Rc<RefCell<Register<u8>>>,
    wy:   Rc<RefCell<Register<u8>>>,
    wx:   Rc<RefCell<Register<u8>>>,
}

impl Block for Control {
    fn reset(&mut self) {}
}

impl Board for Control {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let lcdc = self.lcdc.clone();
        let stat = self.stat.clone();
        let scy = self.scy.clone();
        let scx = self.scx.clone();
        let ly = self.ly.clone();
        let lyc = self.lyc.clone();
        let dma = self.dma.clone();
        let bgp = self.bgp.clone();
        let obp0 = self.obp0.clone();
        let obp1 = self.obp1.clone();
        let wy = self.wy.clone();
        let wx = self.wx.clone();

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
