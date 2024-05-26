//! DMG-01 motherboard.

use log::warn;
use rugby_arch::mem::Ram;
use rugby_arch::mio::Mmio;
use rugby_arch::{Block, Byte, Shared};

use super::noc::Mmap;
use super::ppu::Vram;
use super::soc::Chip;
use crate::api::proc::Processor;

/// Sharp LH5164N (64K SRAM).
pub type Sram = Ram<[Byte; 0x2000]>;

/// Work RAM.
///
/// 8 KiB RAM used as general-purpose transient memory.
type Wram = Sram;

/// DMG-CPU-01 PCB.
#[derive(Debug)]
pub struct Motherboard {
    /// Crystal oscillator.
    pub clk: u128,
    /// Network-on-chip.
    pub noc: Mmap,
    /// System-on-chip.
    pub soc: Chip,
    /// Video RAM.
    pub vram: Shared<Vram>,
    /// Work RAM.
    pub wram: Shared<Wram>,
}

impl Default for Motherboard {
    fn default() -> Self {
        let noc = Mmap::new();
        let vram = Shared::from(Ram::from([Byte::default(); 0x2000]));
        Self {
            clk: u128::default(),
            soc: Chip::new(&noc, vram.clone()),
            noc,
            vram,
            wram: Ram::from([Byte::default(); 0x2000]).into(),
        }
        .prep()
    }
}

impl Motherboard {
    /// Constructs a new `Motherboard`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Prepares a `Motherboard`.
    #[must_use]
    fn prep(self) -> Self {
        self.attach();
        self
    }

    /// Connect memory-mapped I/O to the network-on-chip.
    fn attach(&self) {
        // Borrow network
        let Mmap { ibus, ebus, vbus } = &self.noc;
        let ibus = &mut *ibus.borrow_mut();
        let ebus = &mut *ebus.borrow_mut();
        let vbus = &mut *vbus.borrow_mut();
        // Attach modules
        vbus.map(0x8000..=0x9fff, self.vram.clone().into()); // VRAM
        ebus.map(0xc000..=0xdfff, self.wram.clone().into()); // WRAM
        ebus.map(0xe000..=0xffff, self.wram.clone().into()); // ECHO
        self.soc.attach(ibus);
    }
}

impl Block for Motherboard {
    fn ready(&self) -> bool {
        self.soc.ready()
    }

    fn cycle(&mut self) {
        // Wake on pending interrupt
        if !self.soc.cpu.ready() && self.soc.pic.line.pending() {
            self.soc.cpu.wake();
        }

        // CPU: 1 MiHz
        if self.soc.cpu.ready() && self.clk % 4 == 0 {
            self.soc.cpu.cycle();
        }
        // DMA: 1 MiHz
        if self.soc.dma.ready() && self.clk % 4 == 0 {
            self.soc.dma.cycle();
        }
        // PPU: 4 MiHz
        if self.soc.ppu.ready() {
            self.soc.ppu.cycle();
        }
        // Serial: 8192 Hz
        if self.soc.ser.ready() && self.clk % 512 == 0 {
            self.soc.ser.cycle();
        }
        // Timer: 4 MiHz
        if self.soc.tma.ready() {
            self.soc.tma.cycle();
        }

        // Update executed cycle count
        let (clock, carry) = self.clk.overflowing_add(1);
        if carry {
            warn!("internal cycle counter overflowed; resetting");
        }
        self.clk = clock;
    }

    fn reset(&mut self) {
        self.soc.reset();
    }
}
