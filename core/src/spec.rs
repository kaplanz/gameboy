//! Game Boy physical specs.

use crate::io::joypad::Input;
use crate::io::screen::{Pixel, Resolution, Screen as Lcd};

/// DMG-01 specs.
pub mod dmg {
    use super::*;

    /// Joypad specs.
    pub mod joypad {
        use super::*;

        /// Button layout
        #[rustfmt::skip]
        #[derive(Copy, Clone, Debug)]
        pub enum Button {
            A      = 0b00100001,
            B      = 0b00100010,
            Select = 0b00100100,
            Start  = 0b00101000,
            Right  = 0b00010001,
            Left   = 0b00010010,
            Up     = 0b00010100,
            Down   = 0b00011000,
        }

        impl Input for Button {}
    }

    /// Screen specs.
    pub mod screen {
        use super::*;

        /// Screen data.
        pub type Screen = Lcd<Color, { RES.len() }>;

        /// Resolution info.
        pub const RES: Resolution = Resolution {
            width: 160,
            height: 144,
        };

        /// Color values.
        #[derive(Copy, Clone, Debug, Default)]
        pub enum Color {
            /// Lightest
            #[default]
            C0 = 0b00,
            /// Light Medium
            C1 = 0b01,
            /// Dark Medium
            C2 = 0b10,
            /// Darkest
            C3 = 0b11,
        }

        impl Pixel for Color {}

        impl From<Color> for usize {
            fn from(color: Color) -> Self {
                color as usize
            }
        }
    }
}
