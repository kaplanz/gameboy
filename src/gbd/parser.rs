use std::num::ParseIntError;

use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use super::{Command, Cycle};

#[derive(Parser)]
#[grammar = "gbd/parser.pest"]
struct GbdParser;

#[allow(clippy::cast_sign_loss)]
pub fn parse(src: &str) -> Result<Option<Command>, Error> {
    // Parse the input string
    let mut pairs = GbdParser::parse(Rule::Input, src)?;
    // Extract the top level pair
    let top = pairs.next().expect("missing top rule");

    // Match a command rule
    let cmd = match top.as_rule() {
        Rule::Break => {
            let mut pairs = top.into_inner();
            let addr = parse::int(pairs.next().expect("missing inner rule"))?;
            Command::Break(addr)
        }
        Rule::Continue => Command::Continue,
        Rule::Delete => {
            let mut pairs = top.into_inner();
            let index = parse::int(pairs.next().expect("missing inner rule"))?;
            Command::Delete(index)
        }
        Rule::Freq => {
            let mut pairs = top.into_inner();
            let pair = pairs.next().expect("missing inner rule");
            let cycle = match pair.as_rule() {
                Rule::Dot => Cycle::Dot,
                Rule::Insn => Cycle::Insn,
                Rule::Mach => Cycle::Mach,
                rule => unreachable!("invalid rule: {rule:?}"),
            };
            Command::Freq(cycle)
        }
        Rule::Help => {
            let mut pairs = top.into_inner();
            let what = pairs.next().map(|pair| pair.to_string());
            Command::Help(what)
        }
        Rule::Info => {
            let mut pairs = top.into_inner();
            let what = pairs.next().map(|pair| pair.to_string());
            Command::Info(what)
        }
        Rule::List => Command::List,
        Rule::Quit => Command::Quit,
        Rule::Read => {
            let mut pairs = top.into_inner();
            let what = pairs.next().expect("missing inner rule");
            match what.as_rule() {
                Rule::UInt => {
                    let addr = parse::int(what)?;
                    Command::Read(addr)
                }
                Rule::RangeBounds => {
                    let mut pairs = what.into_inner();
                    let (start, end) = parse::range(pairs.next().expect("missing inner rule"))?;
                    Command::ReadRange(start..end)
                }
                rule => unreachable!("invalid rule: {rule:?}"),
            }
        }
        Rule::Skip => {
            let mut pairs = top.into_inner();
            let index = parse::int(pairs.next().expect("missing inner rule"))?;
            let many = parse::int(pairs.next().expect("missing inner rule"))?;
            Command::Skip(index, many)
        }
        Rule::Step => Command::Step,
        Rule::Write => {
            let mut pairs = top.into_inner();
            let addr = parse::int(pairs.next().expect("missing inner rule"))?;
            let pair = pairs.next().expect("missing inner rule");
            let byte = parse::int(pair.clone()) // attempt both `u8` and `i8`
                .or_else(|_| parse::int::<i8>(pair).map(|int| int as u8))?;
            Command::Write(addr, byte)
        }
        Rule::EOI => return Ok(None),
        rule => unreachable!("invalid rule: {rule:?}"),
    };

    Ok(Some(cmd))
}

mod parse {
    use std::num::ParseIntError;

    use num::traits::WrappingAdd;
    use num::{Bounded, Integer};
    use pest::iterators::Pair;

    use super::Rule;

    pub(super) fn int<I>(pair: Pair<Rule>) -> Result<I, ParseIntError>
    where
        I: Integer<FromStrRadixErr = ParseIntError>,
    {
        // Extract the number and sign
        let mut int = pair
            .into_inner() // `Int` is composed of `Sign` and `Num`
            .rev(); // since sign is optional, reverse to process it last
        let num = int.next().expect("missing inner rule");
        let sign = int.next().map_or("", |rule| rule.as_str()).to_string();
        // Parse into an integer type
        match num.as_rule() {
            Rule::Bin => I::from_str_radix(&(sign + &num.as_str()[2..]), 2),
            Rule::Oct => I::from_str_radix(&(sign + &num.as_str()[2..]), 8),
            Rule::Dec => I::from_str_radix(&(sign + &num.as_str()[0..]), 10),
            Rule::Hex => I::from_str_radix(&(sign + &num.as_str()[2..]), 16),
            rule => unreachable!("invalid rule: {rule:?}"),
        }
    }

    pub(super) fn range<I>(pair: Pair<Rule>) -> Result<(I, I), ParseIntError>
    where
        I: Bounded + Integer<FromStrRadixErr = ParseIntError> + WrappingAdd,
    {
        // Extract the range rule
        match pair.as_rule() {
            Rule::Range => {
                let mut range = pair.into_inner();
                let start = self::int(range.next().expect("missing inner rule"))?;
                let end = self::int(range.next().expect("missing inner rule"))?;
                Ok((start, end))
            }
            Rule::RangeFrom => {
                let mut range = pair.into_inner();
                let start = self::int(range.next().expect("missing inner rule"))?;
                let end = I::max_value();
                Ok((start, end))
            }
            Rule::RangeFull => {
                let start = I::min_value();
                let end = I::max_value();
                Ok((start, end))
            }
            Rule::RangeInc => {
                let mut range = pair.into_inner();
                let start = self::int(range.next().expect("missing inner rule"))?;
                let end = self::int::<I>(range.next().expect("missing inner rule"))?
                    .wrapping_add(&I::one());
                Ok((start, end))
            }
            Rule::RangeTo => {
                let mut range = pair.into_inner();
                let start = I::min_value();
                let end = self::int(range.next().expect("missing inner rule"))?;
                Ok((start, end))
            }
            Rule::RangeToInc => {
                let mut range = pair.into_inner();
                let start = I::min_value();
                let end = self::int::<I>(range.next().expect("missing inner rule"))?
                    .wrapping_add(&I::one());
                Ok((start, end))
            }
            rule => unreachable!("invalid rule: {rule:?}"),
        }
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
