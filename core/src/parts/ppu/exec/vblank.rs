use std::fmt::Display;

use rugby_arch::reg::Register;

use super::hblank::HBlank;
use super::{Mode, Ppu, LCD};

/// Vertical blanking interval.
#[derive(Clone, Debug, Default)]
pub struct VBlank;

impl VBlank {
    /// Number of scanlines of vblank.
    pub const LAST: u16 = LCD.ht + 10;

    pub fn exec(self, ppu: &mut Ppu) -> Mode {
        // Move to next dot
        ppu.etc.dot += 1;

        // Determine next mode
        if ppu.etc.dot < HBlank::DOTS {
            // Continue vblank
            Mode::VBlank(self)
        } else {
            // Increment scanline
            let ly = ppu.reg.ly.load() + 1;
            ppu.reg.ly.store(ly);
            // Reset dot-clock
            ppu.etc.dot = 0;

            // Determine next mode
            if u16::from(ly) < Self::LAST {
                // Continue vblank
                Mode::VBlank(self)
            } else {
                // Reset scanline
                ppu.reg.ly.store(0);
                // Enter scan (next frame)
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
