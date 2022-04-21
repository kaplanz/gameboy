//! Custom devices.
//!
//! The following are specicialized [`Device`](remus::Device) implementations
//! useful for Game Boy emulation.

pub use readonly::ReadOnly;
pub use unmapped::Unmapped;

mod readonly;
mod unmapped;
