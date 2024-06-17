//! Emulation configuration.

use std::path::{Path, PathBuf};

pub use crate::val::Tristate;
use crate::Conf;

/// Emulation options.
#[derive(Debug, Default)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Emulation {
    /// Booting options.
    #[cfg_attr(feature = "clap", clap(flatten))]
    pub boot: Boot,

    /// Cartridge options.
    #[cfg_attr(feature = "clap", clap(flatten))]
    pub cart: Cart,
}

impl Conf for Emulation {
    fn rebase(&mut self, root: &Path) {
        self.boot.rebase(root);
        self.cart.rebase(root);
    }

    fn merge(&mut self, other: Self) {
        self.boot.merge(other.boot);
        self.cart.merge(other.cart);
    }
}

/// Booting options.
#[derive(Debug, Default)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Boot {
    /// Boot ROM image file.
    ///
    /// If the path to the image file is specified within the configuration, it
    /// can be selected by passing `-b/--boot` without specifying an argument.
    /// Otherwise, the path to the image file must be provided.
    ///
    /// May be negated with `--no-boot`.
    #[cfg_attr(feature = "clap", clap(
        name = "boot",
        short, long,
        num_args(0..=1),
        value_name = "PATH",
        value_hint = clap::ValueHint::FilePath,
    ))]
    pub image: Option<PathBuf>,

    /// Skip the boot ROM.
    ///
    /// Negates `-b/--boot`.
    #[cfg_attr(
        feature = "clap",
        clap(
            hide = true,
            long = "no-boot",
            overrides_with = "boot",
            default_value_t = true,
            default_value_if("boot", clap::builder::ArgPredicate::IsPresent, "false"),
        )
    )]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub skip: bool,
}

impl Conf for Boot {
    fn rebase(&mut self, root: &Path) {
        self.image = self.image.take().map(|path| root.join(path));
    }

    fn merge(&mut self, other: Self) {
        self.image = self.image.take().or(other.image);
    }
}

/// Cartridge options.
#[derive(Debug, Default)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Cart {
    /// Cartridge ROM image file.
    ///
    /// A cartridge will be constructed from the data specified in this file.
    /// The cartridge header specifies precisely what hardware will be
    /// instantiated.
    #[cfg_attr(feature = "clap", clap(
        required_unless_present("force"),
        value_hint = clap::ValueHint::FilePath,
        help_heading = None,
    ))]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub rom: Option<PathBuf>,

    /// Check cartridge integrity.
    ///
    /// Verifies that both the header and global checksums match the data within
    /// the ROM.
    #[cfg_attr(feature = "clap", clap(short, long, conflicts_with("force")))]
    pub check: bool,

    /// Force cartridge construction.
    ///
    /// Causes the cartridge generation to always succeed, even if the ROM does
    /// not contain valid data.
    #[cfg_attr(feature = "clap", clap(short, long))]
    pub force: bool,

    /// Cartridge RAM persistence.
    ///
    /// This option can be used to override the cartridge's hardware support for
    /// persistent RAM. When enabled, RAM will be loaded and saved from a file
    /// with the same path and name as the ROM, but using the ".sav" extension.
    #[cfg_attr(feature = "clap", clap(long, value_name = "WHEN", value_enum))]
    pub save: Option<Tristate>,
}

impl Cart {
    /// Cartridge RAM save file.
    ///
    /// The cartridge's RAM be initialized from the data specified in this file.
    pub fn ram(&self) -> Option<PathBuf> {
        self.rom.as_ref().map(|path| path.with_extension("sav"))
    }
}

impl Conf for Cart {
    fn rebase(&mut self, root: &Path) {
        self.rom = self.rom.take().map(|path| root.join(path));
    }

    fn merge(&mut self, other: Self) {
        self.rom = self.rom.take().or(other.rom);
        self.check |= other.check;
        self.force |= other.force;
        self.save = self.save.take().or(other.save);
    }
}
