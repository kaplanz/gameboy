//! Memory-mapped I/O.

use std::cell::RefCell;
use std::rc::Rc;

use crate::mem::Memory;
use crate::Shared;

mod bus;

pub use self::bus::Bus;

/// I/O device.
pub type Device = Shared<dyn Memory>;

impl Device {
    pub fn dev<T: Memory + 'static>(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }
}

impl<M: Memory + 'static> From<Shared<M>> for Device {
    fn from(value: Shared<M>) -> Self {
        Self(value.0)
    }
}

/// Mappable component.
pub trait Mmio {
    /// Attaches this module's devices onto a bus.
    fn attach(&self, bus: &mut Bus);

    /// Detaches this module's devices from a bus.
    ///
    /// # Note
    ///
    /// The default implementation does nothing.
    #[allow(unused_variables)]
    fn detach(&self, bus: &mut Bus) {}
}
