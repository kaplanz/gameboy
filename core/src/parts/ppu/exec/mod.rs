use std::fmt::Display;

use log::trace;
use remus::reg::Register;
use remus::Byte;

use self::draw::Draw;
use self::hblank::HBlank;
use self::scan::Scan;
use self::vblank::VBlank;
use super::meta::sprite;
use super::{blk, Interrupt, Lcdc, Ppu, LCD};

pub mod draw;
pub mod hblank;
pub mod scan;
pub mod vblank;

/// Graphics mode.
#[derive(Clone, Debug)]
pub enum Mode {
    /// Sprite scanning.
    Scan(Scan),
    /// Pixel drawing.
    Draw(Draw),
    /// Horizontal blank.
    HBlank(HBlank),
    /// Vertical blank.
    VBlank(VBlank),
}

impl Mode {
    #[must_use]
    pub(super) fn exec(self, ppu: &mut Ppu) -> Self {
        // Handle previous state
        {
            // Update STAT
            let mut stat = ppu.reg.stat.load();
            let ly = ppu.reg.ly.load();
            let lyc = ppu.reg.lyc.load();
            stat ^= (stat & 0x03) ^ Byte::from(&self);
            stat ^= (stat & 0x04) ^ Byte::from(ly == lyc) << 2;
            ppu.reg.stat.store(stat);

            // Trigger interrupts
            if ppu.etc.dot == 0 {
                let mut int = 0;
                // LYC=LY
                int |= Byte::from(lyc == ly) << 6;
                // Mode 2
                int |= Byte::from(matches!(self, Mode::Scan(_))) << 5;
                // Mode 1
                int |= Byte::from(matches!(self, Mode::VBlank(_))) << 4;
                // Mode 0
                int |= Byte::from(matches!(self, Mode::HBlank(_))) << 3;
                // Check for interrupts
                if int & (stat & 0x78) != 0 {
                    // Request an interrupt
                    ppu.int.raise(Interrupt::LcdStat);
                }
            }
        }

        // Execute the current PPU mode
        trace!("{:03}: {self:?}", ppu.etc.dot);
        match self {
            Mode::Scan(scan) => scan.exec(ppu),
            Mode::Draw(draw) => draw.exec(ppu),
            Mode::HBlank(hblank) => hblank.exec(ppu),
            Mode::VBlank(vblank) => vblank.exec(ppu),
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::Scan(Scan::default())
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scan(scan) => scan.fmt(f),
            Self::Draw(draw) => draw.fmt(f),
            Self::HBlank(hblank) => hblank.fmt(f),
            Self::VBlank(vblank) => vblank.fmt(f),
        }
    }
}

#[rustfmt::skip]
impl From<&Mode> for Byte {
    fn from(mode: &Mode) -> Self {
        match mode {
            Mode::Scan(_)   => 0b10,
            Mode::Draw(_)   => 0b11,
            Mode::HBlank(_) => 0b00,
            Mode::VBlank(_) => 0b01,
        }
    }
}
