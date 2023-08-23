//! Memory ports.

use remus::bus::adapt::View;
use remus::bus::Bus;
use remus::mem::Ram;
use remus::{Block, Board, Device, Shared};

use super::Boot;

pub type Wram = Ram<0x2000>;
pub type Hram = Ram<0x007f>;

/// On-chip memory.
///
/// This struct is plain old data (POD), and its members are accessible by
/// [`GameBoy`](super::GameBoy).
#[derive(Debug, Default)]
pub struct Memory {
    // ┌────────┬──────┬─────┬───────┐
    // │  Size  │ Name │ Dev │ Alias │
    // ├────────┼──────┼─────┼───────┤
    // │  256 B │ Boot │ ROM │       │
    // │  8 KiB │ Work │ RAM │ WRAM  │
    // │  127 B │ High │ RAM │ HRAM  │
    // └────────┴──────┴─────┴───────┘
    boot: Shared<Boot>,
    wram: Shared<Wram>,
    hram: Shared<Hram>,
}

impl Memory {
    /// Constructs a new `Memory` using provided [`Boot`].
    pub fn with(boot: Boot) -> Self {
        Self {
            boot: boot.into(),
            ..Default::default()
        }
    }

    /// Gets the boot ROM.
    #[allow(unused)]
    pub(super) fn boot(&self) -> Shared<Boot> {
        self.boot.clone()
    }

    /// Gets the work RAM.
    pub(super) fn wram(&self) -> Shared<Wram> {
        self.wram.clone()
    }

    /// Gets a view of the echo RAM.
    pub(super) fn echo(&self) -> Shared<impl Device> {
        View::new(self.wram(), 0x0000..=0x1dff).to_shared()
    }

    /// Gets the high RAM.
    pub(super) fn hram(&self) -> Shared<Hram> {
        self.hram.clone()
    }
}

impl Block for Memory {
    fn reset(&mut self) {
        self.boot.reset();
        self.wram.reset();
        self.hram.reset();
    }
}

impl Board for Memory {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Connect boards
        self.boot.borrow().connect(bus);

        // Extract devices
        let wram = self.wram().to_dynamic();
        let echo = self.echo().to_dynamic();
        let hram = self.hram().to_dynamic();

        // Map devices on bus  // ┌──────┬────────┬──────┬─────┐
                               // │ Addr │  Size  │ Name │ Dev │
                               // ├──────┼────────┼──────┼─────┤
        // mapped by `boot`    // │ 0000 │  256 B │ Boot │ ROM │
        bus.map(0xc000, wram); // │ c000 │  8 KiB │ Work │ RAM │
        bus.map(0xe000, echo); // │ e000 │ 7680 B │ Echo │ RAM │
        // mapped by `boot`    // │ ff50 │    1 B │ Boot │ Reg │
        bus.map(0xff80, hram); // │ ff80 │  127 B │ High │ RAM │
                               // └──────┴────────┴──────┴─────┘
    }
}
