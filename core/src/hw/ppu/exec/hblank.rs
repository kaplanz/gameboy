use std::fmt::Display;

use remus::Cell;

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
        ppu.dot += 1;

        // Determine next mode
        if ppu.dot < Self::DOTS {
            // Continue vblank
            Mode::HBlank(self)
        } else {
            // Increment scanline
            let ly = ppu.file.ly.load() + 1;
            ppu.file.ly.store(ly);
            // Reset dot-clock
            ppu.dot = 0;

            // Determine next scanline type
            if u16::from(ly) < LCD.ht {
                // Begin next scanline
                Mode::Scan(self.into())
            } else {
                // Reset internal window line counter
                ppu.winln = 0;
                // Request an interrupt
                ppu.pic.borrow_mut().req(Interrupt::VBlank);
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
