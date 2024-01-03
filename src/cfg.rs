use std::path::PathBuf;

use clap::ValueEnum;
use serde::Deserialize;

use crate::{pal, FREQUENCY};

/// Configuration directory path.
pub fn dir() -> PathBuf {
    dirs::config_dir().unwrap().join("gameboy")
}

/// Emulator configuration.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub gui: Gui,
    pub hw: Hardware,
}

/// Graphical user interface.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Gui {
    #[serde(rename = "palette")]
    pub pal: Palette,
    pub speed: Speed,
}

/// Console hardware description.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Hardware {
    pub boot: Option<PathBuf>,
}

/// Console hardware model.
#[derive(Clone, Debug, Default, Deserialize, ValueEnum)]
pub enum Model {
    /// Game Boy
    #[default]
    Dmg,
}

/// Emulator palette selection.
#[derive(Clone, Debug, Default, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Palette {
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
    Custom(pal::Palette),
}

#[rustfmt::skip]
impl From<Palette> for pal::Palette {
    fn from(value: Palette) -> Self {
        match value {
            Palette::Chrome => pal::CHROME,
            Palette::Legacy => pal::LEGACY,
            Palette::Mystic => pal::MYSTIC,
            Palette::Rustic => pal::RUSTIC,
            Palette::Winter => pal::WINTER,
            Palette::Custom(pal) => pal,
        }
    }
}

/// Emulation speed modifier.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
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
            Speed::Half        => Some(FREQUENCY / 2),
            Speed::Full        => Some(FREQUENCY),
            Speed::Double      => Some(FREQUENCY * 2),
            Speed::Triple      => Some(FREQUENCY * 3),
            Speed::Max         => None,
            Speed::Custom(spd) => Some(spd),
        }
    }
}
