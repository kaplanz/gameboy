//! Register models.

use crate::{Byte, Shared, Word};

/// An access point within a register file.
pub trait Port<R: Register> {
    /// Selection name of register.
    type Select;

    /// Loads from the specified register.
    fn load(&self, reg: Self::Select) -> R;

    /// Stores to the specified register.
    fn store(&mut self, reg: Self::Select, value: R);
}

/// Register load-store interface.
pub trait Register {
    /// Value stored by the register.
    type Value;

    /// Loads the value of a register.
    fn load(&self) -> Self::Value;

    /// Stores a value into a register.
    fn store(&mut self, value: Self::Value);
}

impl Register for Byte {
    type Value = Self;

    fn load(&self) -> Byte {
        *self
    }

    fn store(&mut self, value: Byte) {
        *self = value;
    }
}

impl Register for Word {
    type Value = Self;

    fn load(&self) -> Word {
        *self
    }

    fn store(&mut self, value: Word) {
        *self = value;
    }
}

impl<R: Register> Register for Shared<R> {
    type Value = <R as Register>::Value;

    fn load(&self) -> Self::Value {
        self.borrow().load()
    }

    fn store(&mut self, value: Self::Value) {
        self.borrow_mut().store(value);
    }
}
