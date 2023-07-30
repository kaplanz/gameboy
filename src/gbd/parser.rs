use std::num::ParseIntError;

use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use super::Command;

#[derive(Parser)]
#[grammar = "gbd/parser.pest"]
struct GbdParser;

pub fn parse(src: &str) -> Result<Option<Command>, Error> {
    // Parse the input string
    let mut pairs = GbdParser::parse(Rule::Input, src)?;
    // Extract the top level pair
    let top = pairs.next().expect("missing top rule");

    // Match a command rule
    let cmd = match top.as_rule() {
        Rule::Break => {
            let mut pairs = top.into_inner();
            let addr = parse::int(&pairs.next().expect("missing inner rule"))?;
            Command::Break(addr)
        }
        Rule::Continue => Command::Continue,
        Rule::Delete => {
            let mut pairs = top.into_inner();
            let index = parse::uint(&pairs.next().expect("missing inner rule"))?;
            Command::Delete(index)
        }
        Rule::Help => {
            let mut pairs = top.into_inner();
            let what = pairs.next().map(|pair| pair.to_string());
            Command::Help(what)
        }
        Rule::List => Command::List,
        Rule::Read => {
            let mut pairs = top.into_inner();
            let addr = parse::int(&pairs.next().expect("missing inner rule"))?;
            Command::Read(addr)
        }
        Rule::Write => {
            let mut pairs = top.into_inner();
            let addr = parse::uint(&pairs.next().expect("missing inner rule"))?;
            let byte = parse::int(&pairs.next().expect("missing inner rule"))?;
            Command::Write(addr, byte)
        }
        Rule::Skip => {
            let mut pairs = top.into_inner();
            let index = parse::uint(&pairs.next().expect("missing inner rule"))?;
            let many = parse::uint(&pairs.next().expect("missing inner rule"))?;
            Command::Skip(index, many)
        }
        Rule::Step => Command::Step,
        Rule::EOI => return Ok(None),
        rule => panic!("invalid rule: {rule:?}"),
    };

    Ok(Some(cmd))
}

mod parse {
    use std::num::ParseIntError;
    use std::str::FromStr;

    use num::{Integer, Unsigned};
    use pest::iterators::Pair;

    use super::Rule;

    pub(super) fn int<I>(pair: &Pair<Rule>) -> Result<I, ParseIntError>
    where
        I: Integer + FromStr<Err = ParseIntError>,
    {
        str::parse(pair.as_str())
    }

    pub(super) fn uint<U>(pair: &Pair<Rule>) -> Result<U, ParseIntError>
    where
        U: Unsigned + FromStr<Err = ParseIntError>,
    {
        str::parse(pair.as_str())
    }
}

/// A type specifying categories of [`Debugger`] parse errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
}
