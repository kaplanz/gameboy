//! Debugging the [DMG-01](super).

use super::{cpu, ppu, GameBoy};

/// Gather debug into from the CPU.
pub fn cpu(emu: &mut GameBoy) -> cpu::dbg::Debug {
    cpu::dbg::info(&emu.pcb.soc.cpu)
}

/// Collect debug information from the PPU.
pub fn ppu(emu: &mut GameBoy) -> ppu::dbg::Debug {
    ppu::dbg::info(&emu.pcb.soc.ppu)
}

/// Debug information.
#[derive(Debug)]
pub struct Debug {
    pub cpu: cpu::dbg::Debug,
    pub ppu: ppu::dbg::Debug,
}

impl Debug {
    /// Constructs a new, fully popoulated `Debug`.
    pub fn new(emu: &mut GameBoy) -> Self {
        Self {
            cpu: self::cpu(emu),
            ppu: self::ppu(emu),
        }
    }
}
