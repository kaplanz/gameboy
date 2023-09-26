use log::{debug, trace, warn};
use remus::bus::Bus;
use remus::dev::Device;
use remus::{Address, Block, Cell, Linked, Machine, Shared};

use super::Oam;

const OAM: u8 = 160;

/// Direct memory access.
#[derive(Debug, Default)]
pub struct Dma {
    // State
    page: u8,
    state: State,
    // Shared
    bus: Shared<Bus<u16, u8>>,
    oam: Shared<Oam>,
}

impl Dma {
    /// Constructs a new `Dma`
    pub fn new(bus: Shared<Bus<u16, u8>>, oam: Shared<Oam>) -> Self {
        Self {
            bus,
            oam,
            ..Default::default()
        }
    }
}

impl Address<u16, u8> for Dma {
    fn read(&self, _: u16) -> u8 {
        self.load()
    }

    fn write(&mut self, _: u16, value: u8) {
        self.store(value);
    }
}

impl Block for Dma {
    fn reset(&mut self) {
        // State
        std::mem::take(&mut self.page);
        std::mem::take(&mut self.state);
    }
}

impl Cell<u8> for Dma {
    fn load(&self) -> u8 {
        self.page
    }

    fn store(&mut self, value: u8) {
        match self.state {
            State::Off => {
                // Request a new transfer
                self.state = State::Req(value);
                debug!("request: 0xfe00 <- {:#04x}00", value);
            }
            State::Req(_) | State::On { .. } => {
                warn!("ignored request; already in progress");
            }
        }
        // Always update stored value
        self.page = value;
    }
}

impl Device<u16, u8> for Dma {}

impl Linked<Bus<u16, u8>> for Dma {
    fn mine(&self) -> Shared<Bus<u16, u8>> {
        self.bus.clone()
    }

    fn link(&mut self, it: Shared<Bus<u16, u8>>) {
        self.bus = it;
    }
}

impl Linked<Oam> for Dma {
    fn mine(&self) -> Shared<Oam> {
        self.oam.clone()
    }

    fn link(&mut self, it: Shared<Oam>) {
        self.oam = it;
    }
}

impl Machine for Dma {
    fn enabled(&self) -> bool {
        !matches!(self.state, State::Off)
    }

    fn cycle(&mut self) {
        self.state = match self.state {
            State::Off => {
                panic!("OAM cycled while disabled");
            }
            State::Req(src) => {
                // Initiate transfer
                trace!("started: 0xfe00 <- {:#04x}00", self.page);
                State::On { src, idx: 0 }
            }
            State::On { src, idx } => {
                // Transfer single byte
                let addr = u16::from_be_bytes([src, idx]);
                let data = self.bus.read(addr);
                self.oam.write(idx as usize, data);
                trace!("copied: 0xfe{idx:02x} <- {addr:#06x}, data: {data:#04x}");
                // Increment transfer index
                let idx = idx.saturating_add(1);
                if idx < OAM {
                    State::On { src, idx }
                } else {
                    State::Off
                }
            }
        }
    }
}

/// DMA Transfer State.
#[derive(Debug, Default)]
enum State {
    #[default]
    Off,
    Req(u8),
    On {
        src: u8,
        idx: u8,
    },
}
