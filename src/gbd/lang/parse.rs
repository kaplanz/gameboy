use std::collections::VecDeque;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use super::{Command, Freq, Keyword, Location, Program, Value};

mod imp;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Parser)]
#[grammar = "gbd/lang/parse.pest"]
struct Language;

impl FromStr for Program {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Parse program string
        let pairs = Language::parse(Rule::Program, s)
            .map_err(|err| err.renamed_rules(ToString::to_string))?;
        // Construct program
        #[rustfmt::skip]
        let prog = pairs
            .rev().skip(1).rev() // strip the trailing EOL
            .map(imp::command)   // parse each command
            .collect::<Result<VecDeque<_>>>()
            .map(Self);
        // Handle internal errors
        #[cfg(debug_assertions)]
        if let Err(err @ Error::Internal(_)) = prog.as_ref() {
            panic!("{err}");
        }
        // Return parsed program
        prog
    }
}

#[rustfmt::skip]
#[allow(clippy::enum_glob_use)]
#[allow(clippy::match_same_arms)]
impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Rule::*;

        match self {
            // Keywords
            KBreak    => write!(f, "{Break}"),
            KContinue => write!(f, "{Continue}"),
            KDelete   => write!(f, "{Delete}"),
            KDisable  => write!(f, "{Disable}"),
            KEnable   => write!(f, "{Enable}"),
            KFreq     => write!(f, "{Freq}"),
            KGoto     => write!(f, "{Goto}"),
            KHelp     => write!(f, "{Help}"),
            KIgnore   => write!(f, "{Ignore}"),
            KInfo     => write!(f, "{Info}"),
            KJump     => write!(f, "{Jump}"),
            KList     => write!(f, "{List}"),
            KLoad     => write!(f, "{Load}"),
            KLog      => write!(f, "{Log}"),
            KQuit     => write!(f, "{Quit}"),
            KRead     => write!(f, "{Read}"),
            KReset    => write!(f, "{Reset}"),
            KSerial   => write!(f, "{Serial}"),
            KStep     => write!(f, "{Step}"),
            KStore    => write!(f, "{Store}"),
            KWrite    => write!(f, "{Write}"),
            // Locations
            SerialX   => write!(f, "{Serial}"),
            _ => write!(f, "{self:?}"),
        }
    }
}

/// A type specifying categories of [`Language`] errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("internal error: {0}")]
    Internal(#[from] imp::Error),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),
}
