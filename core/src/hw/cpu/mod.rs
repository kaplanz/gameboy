//! CPU models.
//!
//! The following CPU models may be used within an emulator. To provide a
//! unified interface, all models implement the [`Processor`] trait.

use std::cell::RefCell;
use std::rc::Rc;

use remus::bus::Bus;
use remus::{Block, Machine};

use crate::hw::pic::Pic;

mod sm83;

pub use self::sm83::Cpu as Sm83;

/// Unified processor interface.
pub trait Processor: Block + Machine {
    /// The processor's register set.
    type Register;

    /// Gets the value of the requested register.
    fn get(&self, reg: Self::Register) -> u16;

    /// Sets the value of the requested register.
    fn set(&mut self, reg: Self::Register, value: u16);

    /// Move the PC to the provided address.
    fn goto(&mut self, pc: u16);

    /// Execute the provided instruction in-place.
    fn exec(&mut self, opcode: u8);

    /// Run the provided program (i.e. instruction sequence) in-place.
    fn run(&mut self, prog: &[u8]);

    /// Enable (or wake) the processor.
    fn wake(&mut self);

    /// Sets the processor's memory bus.
    fn set_bus(&mut self, bus: Rc<RefCell<Bus>>);

    /// Sets the processor's interrupt controller.
    fn set_pic(&mut self, pic: Rc<RefCell<Pic>>);
}
