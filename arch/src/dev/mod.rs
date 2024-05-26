//! Devices implementations.

mod null;
mod rand;

pub use self::null::Null;
pub use self::rand::Random;
