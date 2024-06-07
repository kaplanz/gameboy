use log::debug;
use rugby_arch::reg::Register;

use super::hblank::HBlank;
use super::{Mode, Ppu, LCD};

/// Mode 1: Vertical blank.
#[derive(Clone, Debug, Default)]
pub struct VBlank;

impl VBlank {
    /// Number of scanlines of vblank.
    pub const LAST: u16 = LCD.ht + 10;

    pub fn exec(self, ppu: &mut Ppu) -> Mode {
        // Determine next mode
        if ppu.etc.dot + 1 < HBlank::DOTS {
            // Continue vblank
            Mode::VBlank(self)
        } else {
            // Increment scanline
            let ly = ppu.reg.ly.load() + 1;
            ppu.reg.ly.store(ly);

            // Determine next mode
            if u16::from(ly) < Self::LAST {
                // Continue vblank
                Mode::VBlank(self)
            } else {
                // Reset scanline
                ppu.reg.ly.store(0);
                // Enter scan
                debug!("entered mode 2: scan OAM");
                Mode::Scan(self.into())
            }
        }
    }
}

impl From<HBlank> for VBlank {
    fn from(HBlank { .. }: HBlank) -> Self {
        Self
    }
}
