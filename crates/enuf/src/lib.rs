//! Bit modifiable enum flags.
//!
//! `Enuf` should be implemenmted on fieldless enumerations, and uses
//! (usually mutually exclusive) discriminant values as flags.
//!
//! # Examples
//!
//! ```
//! use enuf::Enuf;
//!
//! // Define `Flags` enum
//! #[derive(Copy, Clone)]
//! enum Flag {
//!     A = 0b001, // NOTE: Each of the fields should have descrimenant
//!     B = 0b010, //       values that are mutually exclusive, to ensure
//!     C = 0b100, //       that there is no collision when using flags.
//! }
//!
//! impl Enuf for Flag {}
//!
//! impl From<Flag> for u8 {
//!     fn from(value: Flag) -> Self {
//!         value as u8
//!     }
//! }
//!
//! // Use a `u8` as the flag.
//! let mut flag: u8 = 0;
//!
//! // Set some fields
//! Flag::A.set(&mut flag, false);
//! Flag::B.set(&mut flag, true);
//! Flag::C.set(&mut flag, true);
//!
//! // Read the fields
//! assert_eq!(Flag::A.get(&flag), false);
//! assert_eq!(Flag::B.get(&flag), true);
//! assert_eq!(Flag::C.get(&flag), true);
//! assert_eq!(flag, 0b110);
//! ```

#![warn(clippy::pedantic)]

/// Enum flag interface.
pub trait Enuf: Copy + Into<u8> {
    /// Gets the corresponding bit value from an integer flag.
    fn get(self, f: &u8) -> bool {
        *f & self.into() != 0
    }

    /// Sets the corresponding bit value on an integer flag.
    fn set(self, f: &mut u8, enable: bool) {
        *f ^= (*f & self.into()) ^ (!u8::from(enable).wrapping_sub(1) & self.into());
    }
}
