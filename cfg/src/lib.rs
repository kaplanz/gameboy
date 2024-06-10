//! Emulator configuration.

use std::path::Path;

use crate::opt::{Application, Emulation};

mod val;

pub mod env;
pub mod opt;

/// Configuration interface.
pub trait Conf {
    /// Rebase relative paths to the provided root.
    ///
    /// Any relative paths will have be rebased such that they are not relative
    /// to the provided root.
    fn rebase(&mut self, root: &Path);

    /// Combines two configuration object instances.
    ///
    /// This is useful when some configurations may also be supplied on the
    /// command-line. When merging, it is best practice to prioritize options
    /// from the cli to those saved on-disk. To do so, prefer keeping data
    /// fields from `self` when conflicting with `other`.
    fn merge(&mut self, other: Self);
}

/// Top-level configuration.
#[derive(Debug, Default)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Config {
    /// Application options.
    #[cfg_attr(feature = "clap", clap(flatten, next_help_heading = "Startup"))]
    pub app: Application,

    /// Emulation options.
    #[cfg_attr(feature = "clap", clap(flatten, next_help_heading = "Console"))]
    pub emu: Emulation,
}

impl Conf for Config {
    fn rebase(&mut self, root: &Path) {
        self.app.rebase(root);
        self.emu.rebase(root);
    }

    fn merge(&mut self, other: Self) {
        self.app.merge(other.app);
        self.emu.merge(other.emu);
    }
}
