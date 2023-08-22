use std::fmt::Display;

use super::hblank::HBlank;
use super::{Mode, Ppu, SCREEN};

#[derive(Debug, Default)]
pub struct VBlank;

impl VBlank {
    /// Number of lines for which `VBlank` runs.
    pub const LINES: usize = 10;

    pub fn exec(self, ppu: &mut Ppu) -> Mode {
        // VBlank lasts for 456 dots per scanline
        ppu.dot += 1;
        if ppu.dot < HBlank::DOTS {
            Mode::VBlank(self)
        } else {
            // Extract scanline info
            let ly = &mut **ppu.file.ly.borrow_mut();
            // Increment scanline at the 456th dot, and reset dot-clock
            *ly += 1;
            ppu.dot = 0;

            // VBlank lasts for scanlines 144..154
            if (*ly as usize) < SCREEN.height + Self::LINES {
                Mode::VBlank(self)
            } else {
                // Reset scanline
                *ly = 0;
                // Restart PPU
                Mode::Scan(self.into())
            }
        }
    }
}

impl Display for VBlank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─────────────┐")?;
        writeln!(f, "│ {:^11} │", "VBlank")?;
        write!(f, "└─────────────┘")
    }
}

impl From<HBlank> for VBlank {
    fn from(HBlank { .. }: HBlank) -> Self {
        Self
    }
}
