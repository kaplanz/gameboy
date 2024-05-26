//! Embedded memory blocks.

use rugby_arch::mem::Ram;
use rugby_arch::mio::Mmio;
use rugby_arch::{Byte, Shared};

pub use super::apu::Wave;
pub use super::ppu::Oam;

/// High RAM.
///
/// 127 byte RAM only accessible by the [CPU], used to prevent memory corruption
/// during [DMA].
///
/// [cpu]: super::cpu
/// [dma]: super::dma
pub type Hram = Ram<[Byte; 0x007f]>;

/// Embedded memory.
///
/// |     Address     |  Size  | Name | Description   |
/// |:---------------:|--------|------|---------------|
/// | `$FE00..=$FEA0` |  160 B | OAM  | Object memory |
/// | `$FF30..=$FF3F` |   16 B | WAVE | Wave RAM      |
/// | `$FF80..=$FFFE` |  127 B | HRAM | High RAM      |
#[derive(Debug)]
pub struct Bank {
    /// Object memory.
    pub oam: Shared<Oam>,
    /// Wave memory.
    pub wave: Shared<Wave>,
    /// High RAM.
    pub hram: Shared<Hram>,
}

impl Bank {
    /// Constructs a new `Bank`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Bank {
    fn default() -> Self {
        Self {
            oam: Shared::new(Ram::from([Byte::default(); 0x00a0])),
            wave: Shared::new(Ram::from([Byte::default(); 0x0010])),
            hram: Shared::new(Ram::from([Byte::default(); 0x007f])),
        }
    }
}

impl Mmio for Bank {
    fn attach(&self, bus: &mut rugby_arch::mio::Bus) {
        bus.map(0xfe00..=0xfe9f, self.oam.clone().into());
        bus.map(0xff30..=0xff3f, self.wave.clone().into());
        bus.map(0xff80..=0xfffe, self.hram.clone().into());
    }
}
