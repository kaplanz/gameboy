use std::path::PathBuf;

use clap::{Args, ValueEnum, ValueHint};
use rugby::core::dmg::FREQ;
use rugby::pal;
use serde::Deserialize;

/// Name of this crate.
///
/// This may be used for base subdirectories.
pub const NAME: &str = "rugby";

/// Cartridge options.
#[derive(Args, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Cartridge {
    /// Cartridge ROM image file.
    ///
    /// A cartridge will be constructed from the data specified in the ROM. The
    /// cartridge header specifies precisely what hardware will be instantiated.
    #[clap(required_unless_present("force"))]
    #[clap(value_hint = ValueHint::FilePath)]
    #[clap(help_heading = None)]
    #[serde(skip)]
    pub rom: Option<PathBuf>,

    /// Check cartridge integrity.
    ///
    /// Verifies that both the header and global checksums match the data within
    /// the ROM.
    #[clap(short, long)]
    #[clap(conflicts_with("force"))]
    pub check: bool,

    /// Force cartridge construction.
    ///
    /// Causes the cartridge generation to always succeed, even if the ROM does
    /// not contain valid data.
    #[clap(short, long)]
    #[serde(skip)]
    pub force: bool,
}

impl Cartridge {
    /// Combines two configuration instances.
    ///
    /// This is useful when some configurations may also be supplied on the
    /// command-line. When merging, it is best practice to prioritize options
    /// from the cli to those saved on-disk. To do so, prefer keeping data
    /// fields from `self` when conflicting with `other`.
    pub fn merge(&mut self, other: Self) {
        self.rom = self.rom.take().or(other.rom);
        self.check |= other.check;
        self.force |= other.force;
    }
}

/// Hardware options.
#[derive(Args, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Hardware {
    /// Boot ROM image file.
    ///
    /// Embedded firmware ROM executed upon booting.
    #[clap(short, long, value_name = "PATH")]
    #[clap(value_hint = ValueHint::FilePath)]
    #[clap(help_heading = "Cartridge")]
    pub boot: Option<PathBuf>,
}

impl Hardware {
    /// Combines two configuration instances.
    ///
    /// This is useful when some configurations may also be supplied on the
    /// command-line. When merging, it is best practice to prioritize options
    /// from the cli to those saved on-disk. To do so, prefer keeping data
    /// fields from `self` when conflicting with `other`.
    pub fn merge(&mut self, other: Self) {
        self.boot = self.boot.take().or(other.boot);
    }
}

/// Interface options.
#[derive(Args, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Interface {
    /// DMG color palette.
    ///
    /// Select from a list of preset 2-bit color palettes for the DMG model.
    /// Custom values can be defined in the configuration file.
    #[clap(short, long = "palette", value_name = "COLOR")]
    #[clap(value_enum)]
    #[serde(rename = "palette")]
    pub pal: Option<Palette>,

    /// Simulated clock speed.
    ///
    /// Select from a list of possible speeds to simulate the emulator's clock.
    /// Custom values can be defined in the configuration file.
    #[clap(short, long = "speed", value_name = "FREQ")]
    #[clap(value_enum)]
    #[serde(rename = "speed")]
    pub spd: Option<Speed>,
}

impl Interface {
    /// Combines two configuration instances.
    ///
    /// This is useful when some configurations may also be supplied on the
    /// command-line. When merging, it is best practice to prioritize options
    /// from the cli to those saved on-disk. To do so, prefer keeping data
    /// fields from `self` when conflicting with `other`.
    pub fn merge(&mut self, other: Self) {
        self.pal = self.pal.take().or(other.pal);
        self.spd = self.spd.take().or(other.spd);
    }
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
