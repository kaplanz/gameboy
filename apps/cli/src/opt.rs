//! Application options.

use std::path::{Path, PathBuf};

use clap::builder::ArgPredicate;
use clap::{Args, ValueEnum, ValueHint};
use rugby::core::dmg::FREQ;
use rugby::pal;
use serde::Deserialize;

/// Name of this application.
///
/// This may be used for base subdirectories.
pub const NAME: &str = "rugby";

/// Tristate value.
#[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Tristate {
    /// Never enable.
    Never,
    /// Smartly enable.
    #[default]
    Auto,
    /// Always enable.
    Always,
}

/// Application options.
#[derive(Args, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct General {
    /// DMG color palette.
    ///
    /// Select from a list of preset 2-bit color palettes for the DMG model.
    /// Custom values can be defined in the configuration file.
    #[clap(short, long = "palette")]
    #[clap(value_name = "COLOR")]
    #[clap(value_enum)]
    #[serde(rename = "palette")]
    pub pal: Option<Palette>,

    /// Simulated clock speed.
    ///
    /// Select from a list of possible speeds to simulate the emulator's clock.
    /// Custom values can be defined in the configuration file.
    #[clap(short, long = "speed")]
    #[clap(value_name = "FREQ")]
    #[clap(value_enum)]
    #[serde(rename = "speed")]
    pub spd: Option<Speed>,
}

impl General {
    /// Rebase relative paths to the provided root.
    ///
    /// Any relative paths will have be rebased such that they are not relative
    /// to the provided root.
    #[allow(unused, clippy::unused_self)]
    pub fn rebase(&mut self, root: &Path) {}

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

/// Cartridge options.
#[derive(Args, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Cartridge {
    /// Cartridge ROM image file.
    ///
    /// A cartridge will be constructed from the data specified in this file.
    /// The cartridge header specifies precisely what hardware will be
    /// instantiated.
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
    pub force: bool,

    /// Cartridge RAM persistence.
    ///
    /// This option can be used to override the cartridge's hardware support for
    /// persistent RAM. When enabled, RAM will be loaded and saved from a file
    /// with the same path and name as the ROM, but using the ".sav" extension.
    #[clap(long)]
    #[clap(value_name = "WHEN")]
    #[clap(value_enum)]
    pub save: Option<Tristate>,
}

impl Cartridge {
    /// Cartridge RAM save file.
    ///
    /// The cartridge's RAM be initialized from the data specified in this file.
    pub fn ram(&self) -> Option<PathBuf> {
        self.rom.as_ref().map(|path| path.with_extension("sav"))
    }
}

impl Cartridge {
    /// Rebase relative paths to the provided root.
    ///
    /// Any relative paths will have be rebased such that they are not relative
    /// to the provided root.
    pub fn rebase(&mut self, root: &Path) {
        self.rom = self.rom.take().map(|path| root.join(path));
    }

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
        self.save = self.save.take().or(other.save);
    }
}

/// Hardware options.
#[derive(Args, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Hardware {
    /// Boot ROM options.
    #[clap(flatten)]
    #[serde(flatten)]
    pub boot: BootRom,
}

impl Hardware {
    /// Rebase relative paths to the provided root.
    ///
    /// Any relative paths will have be rebased such that they are not relative
    /// to the provided root.
    pub fn rebase(&mut self, root: &Path) {
        self.boot.rebase(root);
    }

    /// Combines two configuration instances.
    ///
    /// This is useful when some configurations may also be supplied on the
    /// command-line. When merging, it is best practice to prioritize options
    /// from the cli to those saved on-disk. To do so, prefer keeping data
    /// fields from `self` when conflicting with `other`.
    pub fn merge(&mut self, other: Self) {
        self.boot.merge(other.boot);
    }
}

/// Boot ROM options.
#[derive(Args, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BootRom {
    /// Boot ROM image file.
    ///
    /// If the path to the image file is specified within the configuration, it
    /// can be selected by passing `-b/--boot` without specifying an argument.
    /// Otherwise, the path to the image file must be provided.
    ///
    /// May be negated with `--no-boot`.
    #[clap(name = "boot")]
    #[clap(short, long)]
    #[clap(num_args(0..=1))]
    #[clap(value_name = "PATH")]
    #[clap(value_hint = ValueHint::FilePath)]
    #[serde(rename = "boot")]
    pub image: Option<PathBuf>,

    /// Skip the boot ROM.
    ///
    /// Negates `-b/--boot`.
    #[clap(hide = true)]
    #[clap(long = "no-boot")]
    #[clap(overrides_with = "boot")]
    #[clap(default_value_t = true)]
    #[clap(default_value_if("boot", ArgPredicate::IsPresent, "false"))]
    #[serde(skip)]
    pub skip: bool,
}

impl BootRom {
    /// Rebase relative paths to the provided root.
    ///
    /// Any relative paths will have be rebased such that they are not relative
    /// to the provided root.
    pub fn rebase(&mut self, root: &Path) {
        self.image = self.image.take().map(|path| root.join(path));
    }

    /// Combines two configuration instances.
    ///
    /// This is useful when some configurations may also be supplied on the
    /// command-line. When merging, it is best practice to prioritize options
    /// from the cli to those saved on-disk. To do so, prefer keeping data
    /// fields from `self` when conflicting with `other`.
    pub fn merge(&mut self, other: Self) {
        self.image = self.image.take().or(other.image);
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
    /// Maximum possible.
    ///
    /// Unconstrained, limited only by the host system's capabilities.
    Max,
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
