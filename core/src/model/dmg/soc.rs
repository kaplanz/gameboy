//! System-on-chip.

use remus::mio::{Bus, Mmio};
use remus::{Block, Shared};

use super::cpu::Cpu;
use super::joypad::Joypad;
use super::mem::Bank;
use super::noc::Mmap;
use super::pic::Pic;
use super::ppu::{Ppu, Vram};
use super::serial::Serial;
use super::timer::Timer;
use crate::parts::apu::Apu;
use crate::parts::dma::Dma;

/// Sharp LR35902 (DMG-CPU).
#[derive(Debug)]
pub struct Chip {
    /// Audio processing unit.
    pub apu: Apu,
    /// Central processing unit.
    pub cpu: Cpu,
    /// Direct memory access controller.
    pub dma: Dma,
    /// Joypad controller.
    pub joy: Joypad,
    /// Embedded memory.
    pub mem: Bank,
    /// Programmable interrupt controller.
    pub pic: Pic,
    /// Picture processing unit
    pub ppu: Ppu,
    /// Serial communications port.
    pub ser: Serial,
    /// Hardware timer.
    pub tma: Timer,
}

impl Chip {
    /// Constructs a new `SoC`.
    #[must_use]
    pub fn new(noc: &Mmap, vram: Shared<Vram>) -> Self {
        let mem = Bank::new();
        let dma = Dma::new(noc.dma(), mem.oam.clone());
        let pic = Pic::new();
        Self {
            apu: Apu::new(),
            cpu: Cpu::new(noc.cpu(), pic.line.clone()),
            joy: Joypad::new(pic.line.clone()),
            ppu: Ppu::new(vram, mem.oam.clone(), dma.reg.clone(), pic.line.clone()),
            ser: Serial::new(pic.line.clone()),
            tma: Timer::new(pic.line.clone()),
            dma,
            mem,
            pic,
        }
    }
}

impl Block for Chip {
    fn ready(&self) -> bool {
        self.cpu.ready()
    }

    fn reset(&mut self) {
        self.apu.reset();
        self.cpu.reset();
        self.dma.reset();
        self.joy.reset();
        self.ppu.reset();
        self.ser.reset();
        self.tma.reset();
    }
}

impl Mmio for Chip {
    fn attach(&self, bus: &mut Bus) {
        self.apu.attach(bus);
        self.joy.attach(bus);
        self.mem.attach(bus);
        self.pic.attach(bus);
        self.ppu.attach(bus);
        self.ser.attach(bus);
        self.tma.attach(bus);
    }
}
