use std::path::PathBuf;

use crate::NAME;

macro_rules! path {
    ($($dir:tt)*) => {
        $(
            #[doc = concat!("Returns the path to the application's ", stringify!($dir), " directory.")]
            #[must_use]
            pub fn $dir() -> PathBuf {
                xdir::$dir().map(|path| path.join(NAME)).unwrap_or_default()
            }
        )*
    };
}

path! { config }
