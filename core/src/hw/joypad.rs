//! Joypad controller.

use std::cell::RefCell;
use std::rc::Rc;

use log::{info, trace};
use remus::{reg, Block, Device};

use super::pic::{Interrupt, Pic};
use crate::spec::dmg::joypad::Button;

/// Joypad model.
#[rustfmt::skip]
#[derive(Debug, Default)]
pub struct Joypad {
    /// Controller register.
    pub con: Rc<RefCell<Register>>,
    /// Programmable interrupt controller.
    pic: Rc<RefCell<Pic>>,
}

impl Joypad {
    /// Sets the joypad's interrupt controller.
    pub fn set_pic(&mut self, pic: Rc<RefCell<Pic>>) {
        self.pic = pic;
    }

    /// Receives currently pressed buttons.
    pub fn recv(&mut self, btns: Vec<Button>) {
        // Retrieve controller state (inverted)
        let prev = !*self.con.borrow().0;
        let is_empty = btns.is_empty();

        // Calculate updated state
        let next = btns
            // Use `.iter().cloned()` to allow use of `btns` later for logging.
            .iter()
            .cloned()
            // Filter buttons as requested in the controller register
            .filter(|&btn| (prev & btn as u8) & 0x30 != 0)
            // Fold matching pressed buttons' corresponding bits into a byte
            .fold(prev & 0xf0, |acc, btn| acc | ((btn as u8) & 0x0f));

        // Schedule interrupt on updated value
        if (prev & 0x0f) != (next & 0x0f) {
            self.pic.borrow_mut().req(Interrupt::Joypad);
            info!("Input {next:#010b}: {btns:?}"); // log updates with `info`
        } else if !is_empty {
            trace!("Input {next:#010b}: {btns:?}"); // log others with `trace`
        }

        // Update controller state (inverted)
        *self.con.borrow_mut().0 = !next;
    }
}

impl Block for Joypad {
    fn reset(&mut self) {
        // Reset P1
        self.con.borrow_mut().reset();
    }
}

/// Player input register.
#[derive(Debug)]
pub struct Register(reg::Register<u8>);

impl Block for Register {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Default for Register {
    fn default() -> Self {
        Self(reg::Register::from(0xff))
    }
}

impl Device for Register {
    fn contains(&self, index: usize) -> bool {
        self.0.contains(index)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn read(&self, index: usize) -> u8 {
        self.0.read(index)
    }

    fn write(&mut self, index: usize, mut value: u8) {
        // NOTE: Only bits 0x30 are writable
        const MASK: u8 = 0x30;
        let read = self.read(index);
        value = (read & !MASK) | (value & MASK);
        self.0.write(index, value)
    }
}
