//! Debug introspection.

use std::fmt::Display;

use itertools::Itertools;

use super::{ppu, GameBoy};

/// Gather Gameboy Doctor debug info.
///
/// # Panics
///
/// Panics if Gameboy Doctor was not enabled on the emulator.
pub fn doc(emu: &mut GameBoy) -> Doctor {
    emu.doc
        .as_mut()
        .map(std::mem::take)
        .expect("expected doctor to be enabled")
}

/// Gather PPU debug info.
pub fn ppu(emu: &mut GameBoy) -> ppu::Debug {
    emu.ppu().debug()
}

/// Debug information.
#[derive(Debug)]
pub struct Debug {
    pub doc: Doctor,
    pub ppu: ppu::Debug,
}

impl Debug {
    /// Constructs a new, fully popoulated `Debug`.
    pub fn new(emu: &mut GameBoy) -> Self {
        Self {
            doc: doc(emu),
            ppu: ppu(emu),
        }
    }
}

/// Doctor entries.
///
/// An introspecive view of the emulator's state matching the format specified
/// by [Gameboy Doctor][gbdoc].
///
/// [gbdoc]: https://robertheaton.com/gameboy-doctor
#[derive(Debug, Default)]
pub struct Doctor(pub(super) Vec<String>);

impl Display for Doctor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().join("\n").fmt(f)
    }
}
