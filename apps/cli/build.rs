#![allow(dead_code)]

use std::env;
use std::io::{self, Result};
use std::path::PathBuf;

use clap::CommandFactory;
use clap_mangen::Man;

use crate::def::NAME;

#[path = "src/cfg.rs"]
mod cfg;
#[path = "src/cli.rs"]
mod cli;
#[path = "src/def.rs"]
mod def;
#[path = "src/dir.rs"]
mod dir;

/// Manual section.
///
/// The sections of the manual are:
/// 1.  General Commands Manual
/// 2.  System Calls Manual
/// 3.  Library Functions Manual
/// 4.  Kernel Interfaces Manual
/// 5.  File Formats Manual
/// 6.  Games Manual
/// 7.  Miscellaneous Information Manual
/// 8.  System Manager's Manual
/// 9.  Kernel Developer's Manual
const MANSECT: &str = "6";

fn main() -> Result<()> {
    let out = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or(io::ErrorKind::NotFound)?;

    // Generate manual page
    let cmd = cli::Cli::command();
    let man = Man::new(cmd).title(NAME).section(MANSECT);
    man.generate_to(out)?;

    Ok(())
}
