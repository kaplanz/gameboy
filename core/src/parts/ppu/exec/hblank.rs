use std::fmt::Display;

use rugby_arch::reg::Register;

use super::draw::Draw;
use super::{Interrupt, Mode, Ppu, LCD};

/// Horizontal blanking interval.
#[derive(Clone, Debug, Default)]
pub struct HBlank;

impl HBlank {
    /// Maximum number of dots per scanline.
    pub const DOTS: u16 = 456;

    pub fn exec(self, ppu: &mut Ppu) -> Mode {
        // Move to next dot
        ppu.etc.dot += 1;

        // Determine next mode
        if ppu.etc.dot < Self::DOTS {
            // Continue vblank
            Mode::HBlank(self)
        } else {
            // Increment scanline
            let ly = ppu.reg.ly.load() + 1;
            ppu.reg.ly.store(ly);
            // Reset dot-clock
            ppu.etc.dot = 0;

            // Determine next scanline type
            if u16::from(ly) < LCD.ht {
                // Begin next scanline
                Mode::Scan(self.into())
            } else {
                // Reset internal window line counter
                ppu.etc.win = 0;
                // Request an interrupt
                ppu.int.raise(Interrupt::VBlank);
                // Enter vblank
                Mode::VBlank(self.into())
            }
        }
    }
}

impl Display for HBlank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─────────────┐")?;
        writeln!(f, "│ {:^11} │", "HBlank")?;
        write!(f, "└─────────────┘")
    }
}

impl From<Draw> for HBlank {
    fn from(Draw { .. }: Draw) -> Self {
        Self
    }
}
