use log::{debug, trace, warn};
use remus::bus::adapt::Bank;
use remus::bus::Mux;
use remus::dev::Device;
use remus::{reg, Address, Block, Board, Cell, Linked, Machine, Shared};

use super::ppu::Oam;
use crate::dev::{Bus, Unmapped};

const OAM: u8 = 160;

/// Direct memory access.
#[derive(Debug)]
pub struct Dma {
    // Control
    // ┌──────┬────────┬─────┬───────┐
    // │ Size │  Name  │ Dev │ Alias │
    // ├──────┼────────┼─────┼───────┤
    // │  1 B │ Start  │ Reg │ DMA   │
    // └──────┴────────┴─────┴───────┘
    ctl: Shared<Control>,
    // Memory
    oam: Shared<Oam>,
    mem: Shared<Bank<u16, u8>>,
    // Shared
    bus: Shared<Bus>,
}

impl Dma {
    /// Constructs a new `Dma`
    #[must_use]
    pub fn new(bus: Shared<Bus>, oam: Shared<Oam>) -> Self {
        // Construct shared memory
        let mem = {
            let nul = Unmapped::<{ OAM as usize }>::new().to_dynamic();
            let oam = oam.clone().to_dynamic();
            Bank::<u16, u8>::from([oam, nul].as_slice())
        }
        .to_shared();

        Self {
            // Control
            ctl: Shared::new(Control::default()),
            // Memory
            oam,
            mem,
            // Shared
            bus,
        }
    }

    /// Gets a reference to the DMA's control register.
    #[must_use]
    pub fn ctrl(&self) -> Shared<Control> {
        self.ctl.clone()
    }
}

impl Board<u16, u8> for Dma {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let oam = self.mem.clone().to_dynamic();

        // Map devices on bus           // ┌──────┬────────┬────────┬─────┐
                                        // │ Addr │  Size  │  Name  │ Dev │
                                        // ├──────┼────────┼────────┼─────┤
        bus.map(0xfe00..=0xfe9f, oam);  // │ fe00 │  160 B │ Object │ RAM │
                                        // └──────┴────────┴────────┴─────┘
    }
}

impl Block for Dma {
    fn reset(&mut self) {
        self.ctl.reset();
    }
}

impl Linked<Bus> for Dma {
    fn mine(&self) -> Shared<Bus> {
        self.bus.clone()
    }

    fn link(&mut self, it: Shared<Bus>) {
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
        !matches!(self.ctl.borrow().state, State::Off)
    }

    fn cycle(&mut self) {
        let state = match self.ctl.borrow().state {
            State::Off => {
                unreachable!("OAM cycled while disabled");
            }
            State::Req(src) => {
                // Disable OAM
                self.mem.borrow_mut().set(1);
                // Initiate transfer
                trace!("started: 0xfe00 <- {:#04x}00", self.ctl.load());
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
                    // Enable OAM
                    self.mem.borrow_mut().set(0);
                    // Complete transfer
                    debug!("finished");
                    State::Off
                }
            }
        };
        self.ctl.borrow_mut().state = state;
    }
}

/// DMA control register.
#[derive(Debug, Default)]
pub struct Control {
    state: State,
    mpage: reg::Register<u8>,
}

impl Address<u16, u8> for Control {
    fn read(&self, _: u16) -> u8 {
        self.load()
    }

    fn write(&mut self, _: u16, value: u8) {
        self.store(value);
    }
}

impl Block for Control {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Cell<u8> for Control {
    fn load(&self) -> u8 {
        self.mpage.load()
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
        self.mpage.store(value);
    }
}

impl Device<u16, u8> for Control {}

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
