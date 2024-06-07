use log::debug;
use rugby_arch::reg::Register;

use super::draw::Draw;
use super::{Interrupt, Mode, Ppu, LCD};

/// Mode 0: Horizontal blank.
#[derive(Clone, Debug, Default)]
pub struct HBlank;

impl HBlank {
    /// Maximum number of dots per scanline.
    pub const DOTS: u16 = 456;

    pub fn exec(self, ppu: &mut Ppu) -> Mode {
        // Determine next mode
        if ppu.etc.dot + 1 < Self::DOTS {
            // Continue vblank
            Mode::HBlank(self)
        } else {
            // Increment scanline
            let ly = ppu.reg.ly.load() + 1;
            ppu.reg.ly.store(ly);

            // Determine next scanline type
            if u16::from(ly) < LCD.ht {
                // Begin next scanline
                Mode::Scan(self.into())
            } else {
                // Reset internal window line counter
                ppu.etc.ywin = 0;
                // Request an interrupt
                ppu.int.raise(Interrupt::VBlank);
                // Enter vblank
                debug!("entered mode 1: vblank");
                Mode::VBlank(self.into())
            }
        }
    }
}

impl From<Draw> for HBlank {
    fn from(Draw { .. }: Draw) -> Self {
        Self
    }
}
