//! Debug introspection.

use std::fmt::Display;

use itertools::Itertools;

use super::{ppu, GameBoy};

/// Debug builder.
pub struct Builder<'a> {
    emu: &'a mut GameBoy,
    dbg: Debug,
}

impl<'a> Builder<'a> {
    /// Constructs a new `Builder` for [`Debug`] info.
    pub fn new(emu: &'a mut GameBoy) -> Self {
        Self {
            emu,
            dbg: Debug::default()
        }
    }

    /// Finishes construction of the [`Debug`] info.
    pub fn finish(self) -> Debug {
        self.dbg
    }

    /// Populate all fields.
    pub fn all(self) -> Self {
        self.doc().ppu()
    }

    /// Populate Gameboy Doctor debug info.
    pub fn doc(mut self) -> Self {
        self.dbg.doc = self.emu.doc.as_mut().map(std::mem::take);
        self
    }

    /// Populate PPU debug info.
    pub fn ppu(mut self) -> Self {
        self.dbg.ppu = Some(self.emu.ppu.debug());
        self
    }
}

/// Debug information.
///
/// Uses a [`Builder`] for construction.
#[derive(Debug, Default)]
pub struct Debug {
    pub doc: Option<Doctor>,
    pub ppu: Option<ppu::Debug>,
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
