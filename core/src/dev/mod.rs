//! Custom devices.
//!
//! The following are specicialized [`Device`](remus::Device) implementations
//! useful for Game Boy emulation.

mod readonly;
mod unmapped;

pub use self::readonly::ReadOnly;
pub use self::unmapped::Unmapped;

/// 16-bit address byte data bus.
pub type Bus = remus::bus::Bus<u16, u8>;
