//! Direct memory access.

use log::{debug, trace, warn};
use rugby_arch::mem::Memory;
use rugby_arch::mio::Bus;
use rugby_arch::reg::Register;
use rugby_arch::{Block, Byte, Shared, Word};

pub use super::ppu::Oam;

/// Direct memory access unit.
#[derive(Debug)]
pub struct Dma {
    /// DMA register.
    pub reg: Shared<Control>,
    // Memory
    oam: Shared<Oam>,
    // Shared
    bus: Bus,
}

impl Dma {
    /// Constructs a new `Dma`
    #[must_use]
    pub fn new(bus: Bus, oam: Shared<Oam>) -> Self {
        Self {
            // Control
            reg: Shared::default(),
            // Memory
            oam,
            // Shared
            bus,
        }
    }
}

impl Block for Dma {
    fn ready(&self) -> bool {
        !matches!(self.reg.borrow().state, State::Off)
    }

    fn cycle(&mut self) {
        // Determine next state
        let state = match self.reg.borrow().state {
            State::Off => {
                unreachable!("cannot to cycle DMA while disabled");
            }
            State::Req(src) => {
                // FIXME: Disable OAM
                // Initiate transfer
                trace!("started: 0xfe00 <- {src:#04x}00");
                State::On { hi: src, lo: 0x00 }
            }
            State::On { hi, lo } => {
                // Transfer single byte
                let addr = u16::from_be_bytes([hi, lo]);
                let data = self.bus.read(addr).unwrap_or(0xff);
                self.oam.write(lo as Word, data).unwrap();
                trace!("copied: 0xfe{lo:02x} <- {addr:#06x}, data: {data:#04x}");
                // Increment transfer index
                let lo = lo.saturating_add(1);
                if usize::from(lo) < self.oam.borrow().inner().len() {
                    State::On { hi, lo }
                } else {
                    // FIXME: Enable OAM
                    // Complete transfer
                    debug!("finished: 0xfe00 <- {hi:#04x}00");
                    State::Off
                }
            }
        };
        // Update the state
        self.reg.borrow_mut().state = state;
    }

    fn reset(&mut self) {
        self.reg.reset();
    }
}

/// DMA control register.
#[derive(Debug, Default)]
pub struct Control {
    /// DMA progress.
    state: State,
    /// DMA source page.
    mpage: Byte,
}

impl Block for Control {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Memory for Control {
    fn read(&self, _: Word) -> rugby_arch::mem::Result<Byte> {
        Ok(self.load())
    }

    fn write(&mut self, _: Word, data: Byte) -> rugby_arch::mem::Result<()> {
        self.store(data);
        Ok(())
    }
}

impl Register for Control {
    type Value = Byte;

    fn load(&self) -> Self::Value {
        self.mpage.load()
    }

    fn store(&mut self, value: Self::Value) {
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

/// DMA Transfer State.
#[derive(Debug, Default)]
enum State {
    /// Disabled.
    #[default]
    Off,
    /// Requested.
    Req(Byte),
    /// In-progress.
    On { hi: Byte, lo: Byte },
}
