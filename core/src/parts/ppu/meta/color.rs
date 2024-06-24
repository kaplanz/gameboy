use rugby_arch::Byte;

use crate::api::part::video::Pixel;

/// Color values.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Color {
    /// Lightest
    #[default]
    C0 = 0b00,
    /// Light
    C1 = 0b01,
    /// Dark
    C2 = 0b10,
    /// Darkest
    C3 = 0b11,
}

impl From<Byte> for Color {
    fn from(value: Byte) -> Self {
        match value & 0b11 {
            0b00 => Self::C0,
            0b01 => Self::C1,
            0b10 => Self::C2,
            0b11 => Self::C3,
            _ => unreachable!(),
        }
    }
}

impl Pixel for Color {}
