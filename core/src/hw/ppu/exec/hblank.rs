use std::fmt::Display;

use remus::Cell;

use super::draw::Draw;
use super::{Interrupt, Mode, Ppu, SCREEN};

#[derive(Clone, Debug, Default)]
pub struct HBlank;

impl HBlank {
    /// Maximum dot within the scanline for which `HBlank` runs.
    pub const DOTS: usize = 456;

    pub fn exec(self, ppu: &mut Ppu) -> Mode {
        // HBlank lasts until the 456th dot
        ppu.dot += 1;
        if ppu.dot < Self::DOTS {
            Mode::HBlank(self)
        } else {
            // Increment scanline
            let ly = ppu.file.ly.load() + 1;
            ppu.file.ly.store(ly);
            // Reset dot-clock
            ppu.dot = 0;

            // Determine next mode
            if (ly as usize) < SCREEN.height {
                // Begin next scanline
                Mode::Scan(self.into())
            } else {
                // Reset internal window line counter
                ppu.winln = 0;
                // Request an interrupt
                ppu.pic.borrow_mut().req(Interrupt::VBlank);
                // Enter VBlank
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
