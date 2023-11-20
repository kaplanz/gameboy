use remus::bus::adapt::Mask;
use remus::bus::Mux;
use remus::dev::Device;
use remus::Shared;

use crate::arch::Bus;

/// Memory bus architecture.
#[derive(Debug, Default)]
pub(crate) struct NoC {
    /// Internal bus.
    pub int: Shared<Bus>,
    /// External bus.
    pub ext: Shared<Bus>,
    /// Video bus.
    pub vid: Shared<Bus>,
}

impl NoC {
    /// Constructs the memory map for the CPU.
    pub fn cpu(&self) -> Bus {
        // Extract buses
        let ibus = self.int.clone();
        let ebus = self.ext.clone();
        let vbus = self.vid.clone();
        // Order bus layers
        let mut mask = Mask::new();
        mask.push(ibus.clone());
        mask.push(ebus);
        mask.push(vbus);
        // Construct memory map
        let mut mmap = Bus::new();
        mmap.map(0x0000..=0xffff, mask.to_dynamic());
        mmap
    }

    /// Constructs the memory map for the DMA.
    pub fn dma(&self) -> Bus {
        // Extract buses
        let ebus = self.ext.clone();
        let vbus = self.vid.clone();
        // Order bus layers
        let mut mask = Mask::new();
        mask.push(ebus);
        mask.push(vbus);
        // Construct memory map
        let mut mmap = Bus::new();
        mmap.map(0x0000..=0xffff, mask.to_dynamic());
        mmap
    }
}
