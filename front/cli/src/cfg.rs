use std::path::PathBuf;

use clap::ValueEnum;
use gameboy::pal;
use serde::Deserialize;

use crate::FREQUENCY;

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
    #[serde(rename = "speed")]
    pub spd: Speed,
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
    /// Nostalgic autumn sunsets.
    AutumnChill,
    /// Aquatic blues.
    BlkAqu,
    /// Winter snowstorm blues.
    BlueDream,
    /// Combining cool and warm tones.
    Coldfire,
    /// Soft and pastel coral hues.
    Coral,
    /// Cold metallic darks with warm dated plastic lights.
    Demichrome,
    /// Greens and warm browns with an earthy feel.
    Earth,
    /// Creamsicle inspired orange.
    IceCream,
    /// Old-school dot-matrix display.
    Legacy,
    /// Misty forest greens.
    Mist,
    /// Simple blacks and whites.
    #[default]
    Mono,
    /// William Morris's rural palette.
    Morris,
    /// Waterfront at dawn.
    PurpleDawn,
    /// Rusty red and brown hues.
    Rustic,
    /// Deep and passionate purples.
    VelvetCherry,
    /// Whatever colors you want!
    #[allow(unused)]
    #[clap(skip)]
    Custom(pal::Palette),
}

#[rustfmt::skip]
impl From<Palette> for pal::Palette {
    fn from(value: Palette) -> Self {
        match value {
            Palette::AutumnChill  => pal::AUTUMN_CHILL,
            Palette::BlkAqu       => pal::BLK_AQU,
            Palette::BlueDream    => pal::BLUE_DREAM,
            Palette::Coldfire     => pal::COLDFIRE,
            Palette::Coral        => pal::CORAL,
            Palette::Demichrome   => pal::DEMICHROME,
            Palette::Earth        => pal::EARTH,
            Palette::IceCream     => pal::ICE_CREAM,
            Palette::Legacy       => pal::LEGACY,
            Palette::Mist         => pal::MIST,
            Palette::Mono         => pal::MONO,
            Palette::Morris       => pal::MORRIS,
            Palette::PurpleDawn   => pal::PURPLE_DAWN,
            Palette::Rustic       => pal::RUSTIC,
            Palette::VelvetCherry => pal::VELVET_CHERRY,
            Palette::Custom(pal)  => pal,
        }
    }
}

/// Simulated clock frequency.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Speed {
    /// Half speed mode.
    ///
    /// Convenience preset resulting in emulation at half speed.
    Half,
    /// Actual hardware speed.
    ///
    /// Mimics the actual hardware clock frequency. Equal to 4 MiHz (60 FPS).
    #[default]
    Actual,
    /// Double speed mode.
    ///
    /// Convenience preset resulting in emulation at double speed.
    Double,
    /// Frame rate.
    ///
    /// Frequency that targets supplied frame rate (FPS).
    #[clap(skip)]
    #[serde(rename = "fps")]
    Rate(u8),
    /// Clock frequency.
    ///
    /// Precise frequency (Hz) to clock the emulator.
    #[clap(skip)]
    #[serde(rename = "hz")]
    Freq(u32),
    /// Maximum possible.
    ///
    /// Unconstrained, limited only by the host system's capabilities.
    Max,
}

impl Speed {
    /// Converts the `Speed` to it's corresponding frequency.
    #[rustfmt::skip]
    pub fn freq(self) -> Option<u32> {
        match self {
            Speed::Half       => Some(FREQUENCY / 2),
            Speed::Actual     => Some(FREQUENCY),
            Speed::Double     => Some(FREQUENCY * 2),
            Speed::Rate(rate) => Some((FREQUENCY / 60).saturating_mul(rate.into())),
            Speed::Freq(freq) => Some(freq),
            Speed::Max        => None,
        }
    }
}
