//! Custom devices.
//!
//! The following are specicialized [`Device`](remus::Device) implementations
//! useful for Game Boy emulation.

mod readonly;
mod unmapped;

pub use self::readonly::ReadOnly;
pub use self::unmapped::Unmapped;
