use log::{debug, trace};
use remus::bus::Bus;
use remus::{Address, Block, Cell, Device, Machine, Shared};

use super::{Oam, SCREEN};

/// Direct memory access.
#[derive(Debug, Default)]
pub struct Dma {
    // State
    page: u8,
    idx: Option<u8>,
    // Connections
    bus: Shared<Bus>,
    oam: Shared<Oam>,
}

impl Dma {
    pub fn set_bus(&mut self, bus: Shared<Bus>) {
        self.bus = bus;
    }

    pub fn set_oam(&mut self, oam: Shared<Oam>) {
        self.oam = oam;
    }
}

impl Address<u8> for Dma {
    fn read(&self, _: usize) -> u8 {
        self.load()
    }

    fn write(&mut self, _: usize, value: u8) {
        self.store(value);
    }
}

impl Block for Dma {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Cell<u8> for Dma {
    fn load(&self) -> u8 {
        self.page
    }

    fn store(&mut self, value: u8) {
        debug!("Starting DMA @ {value:#04x}00");
        self.page = value;
        self.idx = Some(0);
    }
}

impl Device for Dma {
    fn contains(&self, index: usize) -> bool {
        (0..self.len()).contains(&index)
    }

    fn len(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl Machine for Dma {
    fn enabled(&self) -> bool {
        self.idx.is_some()
    }

    fn cycle(&mut self) {
        // Calculate the address to read from
        let idx = self.idx.as_mut().unwrap();
        let addr = (u16::from(self.page) << 8) | (*idx as u16);
        // Read this byte
        let data = self.bus.read(addr as usize);
        trace!("Transferring OAM({idx:#04x}) <- *{addr:#06x} = {data:#04x}");
        // Write this byte
        self.oam.write(*idx as usize, data);
        // Increment the address
        let idx = *idx + 1;
        self.idx = ((idx as usize) < SCREEN.width).then_some(idx);
    }
}
