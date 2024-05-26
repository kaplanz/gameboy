#![allow(dead_code)]

use std::env;
use std::io::{self, Result};
use std::path::PathBuf;

use clap::{CommandFactory, ValueEnum};
use clap_complete::Shell;
use clap_mangen::Man;

use crate::opt::NAME;

#[path = "src/cfg.rs"]
mod cfg;
#[path = "src/cli.rs"]
mod cli;
#[path = "src/dir.rs"]
mod dir;
#[path = "src/opt.rs"]
mod opt;

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
    // Environment variables
    let out = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or(io::ErrorKind::NotFound)?;
    env::set_var(cli::env::CFG, "");

    // Build clap command
    let cmd = cli::Cli::command();

    // Generate completions
    for shell in Shell::value_variants() {
        clap_complete::generate_to(*shell, &mut cmd.clone(), NAME, &out)?;
    }

    // Generate manual page
    let man = Man::new(cmd).title(NAME).section(MANSECT);
    man.generate_to(&out)?;

    Ok(())
}
