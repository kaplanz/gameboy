use std::num::ParseIntError;

use log::trace;
use num::traits::WrappingAdd;
use num::{Bounded, Integer};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use super::{Command, Cycle, Keyword, Program, Register};

pub fn prog(src: &str) -> Result<Program, Error> {
    Language::prog(src)
        .map(IntoIterator::into_iter)
        .map(Program::new)
}

#[derive(Debug, Parser)]
#[grammar = "gbd/lang/parse.pest"]
struct Language;

impl Language {
    fn prog(src: &str) -> Result<Vec<Command>, Error> {
        // Parse program string
        let pairs = Language::parse(Rule::Program, src)?;
        // Extract individual commands
        pairs
            .filter(|pair| !matches!(pair.as_rule(), Rule::EOI))
            .map(Self::cmd)
            .collect()
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::similar_names)]
    #[allow(clippy::too_many_lines)]
    fn cmd(input: Pair<Rule>) -> Result<Command, Error> {
        // Extract keyword and args
        let rule = input.as_rule();
        let mut args = input.into_inner();
        let kword = args.next().ok_or(Error::ExpectedRule)?;
        trace!(
            "keyword: {kword:?}, args: {args:?}",
            kword = kword.as_rule(),
            args = args.clone().map(|arg| arg.as_rule()).collect::<Vec<_>>()
        );

        // Parse individual command
        let cmd = match rule {
            Rule::Break => {
                let addr = Self::int(args.next().ok_or(Error::ExpectedRule)?)?;
                Command::Break(addr)
            }
            Rule::Continue => Command::Continue,
            Rule::Delete => {
                let index = Self::int(args.next().ok_or(Error::ExpectedRule)?)?;
                Command::Delete(index)
            }
            Rule::Disable => {
                let index = Self::int(args.next().ok_or(Error::ExpectedRule)?)?;
                Command::Disable(index)
            }
            Rule::Enable => {
                let index = Self::int(args.next().ok_or(Error::ExpectedRule)?)?;
                Command::Enable(index)
            }
            Rule::Freq => {
                let pair = args.next().ok_or(Error::ExpectedRule)?;
                let cycle = match pair.as_rule() {
                    Rule::Dot => Cycle::Dot,
                    Rule::Insn => Cycle::Insn,
                    Rule::Mach => Cycle::Mach,
                    rule => unreachable!("invalid rule: {rule:?}"),
                };
                Command::Freq(cycle)
            }
            Rule::Help => {
                let what = args.next().map(Self::kword).transpose()?;
                Command::Help(what)
            }
            Rule::Info => {
                let what = args.next().map(Self::kword).transpose()?;
                Command::Info(what)
            }
            Rule::Jump => {
                let addr = Self::int(args.next().ok_or(Error::ExpectedRule)?)?;
                Command::Jump(addr)
            }
            Rule::List => Command::List,
            Rule::Load => {
                let reg = Self::reg(args.next().ok_or(Error::ExpectedRule)?)?;
                Command::Load(reg)
            }
            Rule::Log => {
                let filter = args.next().map(|pair| pair.as_span().as_str().to_string());
                Command::Log(filter)
            }
            Rule::Quit => Command::Quit,
            Rule::Read => {
                let what = args.next().ok_or(Error::ExpectedRule)?;
                // Match on address (range)
                match what.as_rule() {
                    Rule::UInt => {
                        let addr = Self::int(what)?;
                        Command::Read(addr)
                    }
                    Rule::RangeBounds => {
                        let mut pairs = what.into_inner();
                        let (start, end) = Self::range(pairs.next().ok_or(Error::ExpectedRule)?)?;
                        Command::ReadRange(start..end)
                    }
                    rule => unreachable!("invalid rule: {rule:?}"),
                }
            }
            Rule::Reset => Command::Reset,
            Rule::Skip => {
                let index = Self::int(args.next().ok_or(Error::ExpectedRule)?)?;
                let many = Self::int(args.next().ok_or(Error::ExpectedRule)?)?;
                Command::Skip(index, many)
            }
            Rule::Step => {
                let many = args.next().map(Self::int).transpose()?;
                Command::Step(many)
            }
            Rule::Store => {
                let reg = Self::reg(args.next().ok_or(Error::ExpectedRule)?)?;
                let word = Self::int(args.next().ok_or(Error::ExpectedRule)?)?;
                Command::Store(reg, word)
            }
            Rule::Write => {
                let what = args.next().ok_or(Error::ExpectedRule)?;
                // Match on data byte
                let pair = args.next().ok_or(Error::ExpectedRule)?;
                let byte = Self::int(pair.clone()) // attempt both `u8` and `i8`
                    .or_else(|_| Self::int::<i8>(pair).map(|int| int as u8))?;
                // Match on address (range)
                match what.as_rule() {
                    Rule::UInt => {
                        let addr = Self::int(what)?;
                        Command::Write(addr, byte)
                    }
                    Rule::RangeBounds => {
                        let mut pairs = what.into_inner();
                        let (start, end) = Self::range(pairs.next().ok_or(Error::ExpectedRule)?)?;
                        Command::WriteRange(start..end, byte)
                    }
                    rule => unreachable!("invalid rule: {rule:?}"),
                }
            }
            rule => unreachable!("invalid rule: {rule:?}"),
        };

        Ok(cmd)
    }

    fn int<I>(pair: Pair<Rule>) -> Result<I, Error>
    where
        I: Integer<FromStrRadixErr = ParseIntError>,
    {
        // Extract the number and sign
        let mut int = pair
            .into_inner() // `Int` is composed of `Sign` and `Num`
            .rev(); // since sign is optional, reverse to process it last
        let num = int.next().ok_or(Error::ExpectedRule)?;
        let sign = int.next().map_or("", |rule| rule.as_str()).to_string();
        // Parse into an integer type
        match num.as_rule() {
            Rule::Bin => I::from_str_radix(&(sign + &num.as_str()[2..]), 2),
            Rule::Oct => I::from_str_radix(&(sign + &num.as_str()[2..]), 8),
            Rule::Dec => I::from_str_radix(&(sign + &num.as_str()[0..]), 10),
            Rule::Hex => I::from_str_radix(&(sign + &num.as_str()[2..]), 16),
            rule => unreachable!("invalid rule: {rule:?}"),
        }
        .map_err(Error::ParseInt)
    }

    fn range<I>(pair: Pair<Rule>) -> Result<(I, I), Error>
    where
        I: Bounded + Integer<FromStrRadixErr = ParseIntError> + WrappingAdd,
    {
        // Extract the range rule
        match pair.as_rule() {
            Rule::Range => {
                let mut range = pair.into_inner();
                let start = Self::int(range.next().ok_or(Error::ExpectedRule)?)?;
                let end = Self::int(range.next().ok_or(Error::ExpectedRule)?)?;
                Ok((start, end))
            }
            Rule::RangeFrom => {
                let mut range = pair.into_inner();
                let start = Self::int(range.next().ok_or(Error::ExpectedRule)?)?;
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
                let start = Self::int(range.next().ok_or(Error::ExpectedRule)?)?;
                let end = Self::int::<I>(range.next().ok_or(Error::ExpectedRule)?)?
                    .wrapping_add(&I::one());
                Ok((start, end))
            }
            Rule::RangeTo => {
                let mut range = pair.into_inner();
                let start = I::min_value();
                let end = Self::int(range.next().ok_or(Error::ExpectedRule)?)?;
                Ok((start, end))
            }
            Rule::RangeToInc => {
                let mut range = pair.into_inner();
                let start = I::min_value();
                let end = Self::int::<I>(range.next().ok_or(Error::ExpectedRule)?)?
                    .wrapping_add(&I::one());
                Ok((start, end))
            }
            rule => unreachable!("invalid rule: {rule:?}"),
        }
    }

    #[rustfmt::skip]
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::unnecessary_wraps)]
    fn kword(pair: Pair<Rule>) -> Result<Keyword, Error> {
        // Extract the keyword rule
        Ok(match pair.as_rule() {
            Rule::KBreak    => Keyword::Break,
            Rule::KContinue => Keyword::Continue,
            Rule::KDelete   => Keyword::Delete,
            Rule::KDisable  => Keyword::Disable,
            Rule::KEnable   => Keyword::Enable,
            Rule::KFreq     => Keyword::Freq,
            Rule::KHelp     => Keyword::Help,
            Rule::KInfo     => Keyword::Info,
            Rule::KJump     => Keyword::Jump,
            Rule::KList     => Keyword::List,
            Rule::KLoad     => Keyword::Load,
            Rule::KLog      => Keyword::Log,
            Rule::KQuit     => Keyword::Quit,
            Rule::KRead     => Keyword::Read,
            Rule::KReset    => Keyword::Reset,
            Rule::KSkip     => Keyword::Skip,
            Rule::KStep     => Keyword::Step,
            Rule::KStore    => Keyword::Store,
            Rule::KWrite    => Keyword::Write,
            rule => unreachable!("invalid rule: {rule:?}"),
        })
    }

    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::unnecessary_wraps)]
    fn reg(pair: Pair<Rule>) -> Result<Register, Error> {
        // Extract the register rule
        Ok(match pair.as_rule() {
            Rule::AF => Register::AF,
            Rule::BC => Register::BC,
            Rule::DE => Register::DE,
            Rule::HL => Register::HL,
            Rule::SP => Register::SP,
            Rule::PC => Register::PC,
            rule => unreachable!("invalid rule: {rule:?}"),
        })
    }
}

/// A type specifying categories of [`Language`] parse errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("expected rule")]
    ExpectedRule,
}
