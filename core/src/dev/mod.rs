//! Custom devices.
//!
//! The following are specicialized [`Device`](remus::Device) implementations
//! useful for Game Boy emulation.

mod readonly;
mod unmapped;

pub(crate) use self::readonly::ReadOnly;
pub(crate) use self::unmapped::Unmapped;
