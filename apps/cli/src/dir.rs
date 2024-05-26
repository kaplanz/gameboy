//! Standard directories.

use std::path::PathBuf;

use crate::opt::NAME;

/// Convenience macro for creating standard directory definition functions.
macro_rules! path {
    ($($dir:tt)*) => {
        $(
            #[allow(unused)]
            #[doc = concat!("Returns the path to the application's ", stringify!($dir), " directory.")]
            #[must_use]
            pub fn $dir() -> PathBuf {
                xdir::$dir().map(|path| path.join(NAME)).unwrap_or_default()
            }
        )*
    };
}

path! { config state }
