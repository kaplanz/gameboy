//! Joypad controller.

use std::cell::RefCell;
use std::rc::Rc;

use log::{debug, trace};
use remus::bus::Bus;
use remus::{reg, Address, Block, Board, Cell, Device, Shared};

use super::pic::{Interrupt, Pic};

/// Joypad buttons.
#[rustfmt::skip]
#[derive(Copy, Clone, Debug)]
pub enum Button {
    A      = 0b0010_0001,
    B      = 0b0010_0010,
    Select = 0b0010_0100,
    Start  = 0b0010_1000,
    Right  = 0b0001_0001,
    Left   = 0b0001_0010,
    Up     = 0b0001_0100,
    Down   = 0b0001_1000,
}

/// Joypad model.
#[rustfmt::skip]
#[derive(Debug, Default)]
pub struct Joypad {
    // Connections
    pic: Rc<RefCell<Pic>>,
    // Devices
    // ┌──────┬────────┬─────┬───────┐
    // │ Size │  Name  │ Dev │ Alias │
    // ├──────┼────────┼─────┼───────┤
    // │  1 B │ Joypad │ Reg │ CON   │
    // └──────┴────────┴─────┴───────┘
    con: Shared<Control>,
}

impl Joypad {
    /// Gets a reference to the joypad's control register.
    #[must_use]
    pub fn con(&self) -> Shared<Control> {
        self.con.clone()
    }

    /// Sets the joypad's interrupt controller.
    pub fn set_pic(&mut self, pic: Rc<RefCell<Pic>>) {
        self.pic = pic;
    }

    /// Handle pressed button inputs.
    pub fn input(&mut self, keys: &[Button]) {
        // Retrieve controller state (inverted)
        let prev = !self.con.load();
        let is_empty = keys.is_empty();

        // Calculate updated state
        let next = keys
            // Use `.iter().copied()` to allow use of `btns` later for logging.
            .iter()
            .copied()
            // Filter buttons as requested in the controller register
            .filter(|&btn| (prev & btn as u8) & 0x30 != 0)
            // Fold matching pressed buttons' corresponding bits into a byte
            .fold(prev & 0xf0, |acc, btn| acc | ((btn as u8) & 0x0f));

        // Schedule interrupt on updated value
        if (prev & 0x0f) != (next & 0x0f) {
            self.pic.borrow_mut().req(Interrupt::Joypad);
            debug!("Input {next:#010b}: {keys:?}"); // log updates with `debug`
        } else if !is_empty {
            trace!("Input {next:#010b}: {keys:?}"); // log others with `trace`
        }

        // Update controller state (inverted)
        self.con.borrow_mut().0.store(!next);
    }
}

impl Block for Joypad {
    fn reset(&mut self) {
        self.con.reset();
    }
}

impl Board for Joypad {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let con = self.con().to_dynamic();

        // Map devices on bus // ┌──────┬──────┬────────┬─────┐
                              // │ Addr │ Size │  Name  │ Dev │
                              // ├──────┼──────┼────────┼─────┤
        bus.map(0xff00, con); // │ ff00 │  1 B │ Joypad │ Reg │
                              // └──────┴──────┴────────┴─────┘
    }
}

/// Player input register.
#[derive(Debug)]
pub struct Control(reg::Register<u8>);

impl Address<u8> for Control {
    fn read(&self, _: usize) -> u8 {
        self.load()
    }

    fn write(&mut self, _: usize, value: u8) {
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
        self.0.load()
    }

    fn store(&mut self, mut value: u8) {
        // NOTE: Only bits masked bits are writable
        const MASK: u8 = 0b0011_0000;
        value = (value & MASK) | (self.load() & !MASK);
        self.0.store(value);
    }
}

impl Default for Control {
    fn default() -> Self {
        Self(reg::Register::from(0xff))
    }
}

impl Device for Control {
    fn contains(&self, index: usize) -> bool {
        self.0.contains(index)
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
