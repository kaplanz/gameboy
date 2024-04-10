use remus::bus::Mux;
use remus::dev::Device;
use remus::{Block, Board, Shared};

use super::cpu::Cpu;
use super::noc::NoC;
use super::pic::Pic;
use super::ppu::{Oam, Ppu, Vram};
use super::{boot, Hram};
use crate::dev::{Bus, Unmapped};
use crate::hw::apu::Apu;
use crate::hw::dma::Dma;

/// Sharp LR35902 system-on-chip.
#[derive(Debug)]
pub struct SoC {
    // Processors
    pub apu: Apu,
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub dma: Dma,
    // Memory
    pub boot: Option<boot::Rom>,
    pub hram: Shared<Hram>,
}

impl SoC {
    /// Constructs a new `SoC`.
    pub fn new(vram: Shared<Vram>, noc: &NoC, pic: Shared<Pic>) -> Self {
        // Create memory maps
        let cpu = noc.cpu().to_shared();
        let dma = noc.dma().to_shared();
        // Create shared memory
        let oam = Shared::new(Oam::default());
        // Create shared blocks
        let dma = Dma::new(dma, oam.clone());

        // Construct self
        Self {
            // Processors
            apu: Apu::default(),
            cpu: Cpu::new(cpu, pic.clone()),
            ppu: Ppu::new(vram, oam, dma.ctrl(), pic),
            dma,
            // Memory
            boot: Option::default(),
            hram: Shared::new(Hram::default()),
        }
    }
}

impl Block for SoC {
    fn reset(&mut self) {
        // Processors
        self.apu.reset();
        self.cpu.reset();
        self.ppu.reset();
        // Memory
        if let Some(boot) = &mut self.boot {
            boot.reset();
        }
        self.hram.reset();
    }
}

impl Board<u16, u8> for SoC {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Connect boards
        self.apu.connect(bus);
        self.cpu.connect(bus);
        self.dma.connect(bus);
        self.ppu.connect(bus);

        // Extract devices
        let hram = self.hram.clone().to_dynamic();

        // Map devices on bus           // ┌──────┬────────┬──────┬─────┐
                                        // │ Addr │  Size  │ Name │ Dev │
                                        // ├──────┼────────┼──────┼─────┤
        // mapped by `boot`             // │ 0000 │  256 B │ Boot │ ROM │
        // mapped by `boot`             // │ ff50 │    1 B │ Boot │ Reg │
        bus.map(0xff80..=0xfffe, hram); // │ ff80 │  127 B │ High │ RAM │
                                        // └──────┴────────┴──────┴─────┘

        // Memory
        if let Some(boot) = &self.boot {
            boot.connect(bus);
        }

        // Report unmapped reads as `0xff`
        let unmap = Unmapped::<0x200>::new().to_dynamic();
        bus.map(0xfe00..=0xffff, unmap);
    }
}
