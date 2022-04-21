//! CPU models.
//!
//! The following CPU models may be used within an emulator. To provide a
//! unified interface, all models implement the [`Processor`] trait.

use std::cell::RefCell;
use std::rc::Rc;

use remus::bus::Bus;
use remus::{Block, Machine};

pub use self::sm83::Cpu as Sm83;
use crate::hw::pic::Pic;

mod sm83;

/// Unified processor interface.
pub trait Processor: Block + Machine {
    /// Sets the processor's memory bus.
    fn set_bus(&mut self, bus: Rc<RefCell<Bus>>);

    /// Sets the processor's interrupt controller.
    fn set_pic(&mut self, pic: Rc<RefCell<Pic>>);

    /// Enable (or wake) the processor.
    fn wake(&mut self);
}
