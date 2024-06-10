//! Configurable values.

use rugby::core::dmg::FREQ;
use rugby::pal;

/// Tristate value.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub enum Tristate {
    /// Never enable.
    Never,
    /// Smartly enable.
    #[default]
    Auto,
    /// Always enable.
    Always,
}

/// Emulator palette selection.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
#[non_exhaustive]
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
    #[cfg_attr(feature = "clap", clap(skip))]
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
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
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
    /// Maximum possible.
    ///
    /// Unconstrained, limited only by the host system's capabilities.
    Max,
    /// Frame rate.
    ///
    /// Frequency that targets supplied frame rate (FPS).
    #[cfg_attr(feature = "clap", clap(skip))]
    #[cfg_attr(feature = "serde", serde(rename = "fps"))]
    Rate(u8),
    /// Clock frequency.
    ///
    /// Precise frequency (Hz) to clock the emulator.
    #[cfg_attr(feature = "clap", clap(skip))]
    #[cfg_attr(feature = "serde", serde(rename = "hz"))]
    Freq(u32),
}

impl Speed {
    /// Converts the `Speed` to its corresponding frequency.
    #[rustfmt::skip]
    pub fn freq(self) -> Option<u32> {
        match self {
            Speed::Half       => Some(FREQ / 2),
            Speed::Actual     => Some(FREQ),
            Speed::Double     => Some(FREQ * 2),
            Speed::Rate(rate) => Some((FREQ / 60).saturating_mul(rate.into())),
            Speed::Freq(freq) => Some(freq),
            Speed::Max        => None,
        }
    }
}
