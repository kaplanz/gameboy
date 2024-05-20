//! Game Boy Architecture

#![warn(clippy::pedantic)]

mod blk;
mod clk;

pub mod dev;
pub mod mem;
pub mod mio;
pub mod reg;

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub use crate::blk::Block;
pub use crate::clk::Clock;

/// Native architecture byte.
pub type Byte = u8;
/// Native architecture word.
pub type Word = u16;

/// Shared memory-mapped device.
#[derive(Debug, Default)]
pub struct Shared<T: ?Sized>(Inner<T>);

/// Underlying shared pointer.
type Inner<T> = Rc<RefCell<T>>;

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }
}

impl<T: ?Sized> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T: ?Sized> Deref for Shared<T> {
    type Target = Inner<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ?Sized> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }
}

impl<T: ?Sized> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(self, other)
    }
}
