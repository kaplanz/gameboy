//! Memory ports.

use std::cell::RefCell;
use std::rc::Rc;

use remus::bus::adapt::View;
use remus::bus::Bus;
use remus::mem::Ram;
use remus::{Block, Device};

use super::boot::Rom as BootRom;
use super::Board;

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
    pub(super) boot: Rc<RefCell<BootRom>>,
    pub(super) wram: Rc<RefCell<Wram>>,
    pub(super) hram: Rc<RefCell<Hram>>,
}

impl Block for Memory {
    fn reset(&mut self) {
        self.boot.borrow_mut().reset();
        self.wram.borrow_mut().reset();
        self.hram.borrow_mut().reset();
    }
}

impl Board for Memory {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Connect boards
        self.boot.borrow().connect(bus);

        // Extract devices
        let wram = self.wram.clone();
        let echo = View::new(wram.clone(), 0x0000..=0x1dff).to_shared();
        let hram = self.hram.clone();

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
