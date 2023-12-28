use clap::ValueEnum;

use crate::pal::{self, Palette};
use crate::FREQ;

/// Emulation configuration.
#[derive(Debug)]
pub struct Settings {
    /// Palette.
    pub pal: Palette,
    /// Frequency.
    pub spd: Option<u32>,
}

/// Emulator palette selection.
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum Theme {
    /// Chrome palette.
    #[default]
    Chrome,
    /// Legacy palette.
    Legacy,
    /// Mystic palette.
    Mystic,
    /// Rustic palette.
    Rustic,
    /// Winter palette.
    Winter,
    /// Custom palette.
    #[allow(unused)]
    #[clap(skip)]
    Custom(Palette),
}

#[rustfmt::skip]
impl From<Theme> for Palette {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Chrome => pal::CHROME,
            Theme::Legacy => pal::LEGACY,
            Theme::Mystic => pal::MYSTIC,
            Theme::Rustic => pal::RUSTIC,
            Theme::Winter => pal::WINTER,
            Theme::Custom(pal) => pal,
        }
    }
}

/// Emulation speed modifier.
#[derive(Clone, Debug, Default, PartialEq, ValueEnum)]
pub enum Speed {
    /// 30 fps.
    Half,
    /// 60 fps.
    #[default]
    Full,
    /// 120 fps.
    Double,
    /// 180 fps.
    Triple,
    /// Maximum possible.
    Max,
    /// Custom frequency.
    #[allow(unused)]
    #[clap(skip)]
    Custom(u32),
}

#[rustfmt::skip]
impl From<Speed> for Option<u32> {
    fn from(value: Speed) -> Self {
        match value {
            Speed::Half        => Some(FREQ / 2),
            Speed::Full        => Some(FREQ),
            Speed::Double      => Some(FREQ * 2),
            Speed::Triple      => Some(FREQ * 3),
            Speed::Max         => None,
            Speed::Custom(spd) => Some(spd),
        }
    }
}
