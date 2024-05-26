//! Memory mapped device.
//!
//! # Usage
//!
//! Devices are mostly useful in combination with [`Bus`](crate::bus::Bus), with
//! which they can be used to emulate the behaviour of [memory-mapped I/O].
//!
//! [memory-mapped I/O]: https://en.wikipedia.org/wiki/Memory-mapped_I/O

mod null;
mod rand;

pub use self::null::Null;
pub use self::rand::Random;
