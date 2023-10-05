use std::fmt::Display;
use std::num::ParseIntError;

use gameboy::core::dmg::{cpu, pic, ppu, serial, timer};
use log::trace;
use num::traits::WrappingAdd;
use num::{Bounded, Integer};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use super::{Command, Keyword, Location, Mode, Program, Value};

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
        let pairs = Language::parse(Rule::Program, src)
            .map_err(|err| err.renamed_rules(|rule| format!("{rule}")))?;
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
                let mode = match pair.as_rule() {
                    Rule::Dot => Mode::Dot,
                    Rule::Mach => Mode::Mach,
                    Rule::Insn => Mode::Insn,
                    rule => unreachable!("invalid rule: {rule:?}"),
                };
                Command::Freq(mode)
            }
            Rule::Goto => {
                let addr = Self::int(args.next().ok_or(Error::ExpectedRule)?)?;
                Command::Goto(addr)
            }
            Rule::Help => {
                let what = args.next().map(Self::kword).transpose()?;
                Command::Help(what)
            }
            Rule::Ignore => {
                let index = Self::int(args.next().ok_or(Error::ExpectedRule)?)?;
                let many = Self::int(args.next().ok_or(Error::ExpectedRule)?)?;
                Command::Ignore(index, many)
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
                let loc = Self::loc(args.next().ok_or(Error::ExpectedRule)?)?;
                Command::Load(loc)
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
            Rule::Serial => Command::Serial,
            Rule::Step => {
                let many = args.next().map(Self::int).transpose()?;
                Command::Step(many)
            }
            Rule::Store => {
                let loc = Self::loc(args.next().ok_or(Error::ExpectedRule)?)?;
                let value = args.next().ok_or(Error::ExpectedRule)?;
                let value = match loc {
                    Location::Byte(_) | Location::Pic(_) | Location::Ppu(_) | Location::Serial(_) | Location::Timer(_) => Value::Byte(
                        Self::int(value.clone()) // attempt both `u8` and `i8`
                            .or_else(|_| Self::int::<i8>(value).map(|int| int as u8))?,
                    ),
                    Location::Word(_) => Value::Word(
                        Self::int(value.clone()) // attempt both `u16` and `i16`
                            .or_else(|_| Self::int::<i16>(value).map(|int| int as u16))?,
                    ),
                };
                Command::Store(loc, value)
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
            Rule::KGoto     => Keyword::Goto,
            Rule::KHelp     => Keyword::Help,
            Rule::KIgnore   => Keyword::Ignore,
            Rule::KInfo     => Keyword::Info,
            Rule::KJump     => Keyword::Jump,
            Rule::KList     => Keyword::List,
            Rule::KLoad     => Keyword::Load,
            Rule::KLog      => Keyword::Log,
            Rule::KQuit     => Keyword::Quit,
            Rule::KRead     => Keyword::Read,
            Rule::KReset    => Keyword::Reset,
            Rule::KSerial   => Keyword::Serial,
            Rule::KStep     => Keyword::Step,
            Rule::KStore    => Keyword::Store,
            Rule::KWrite    => Keyword::Write,
            rule => unreachable!("invalid rule: {rule:?}"),
        })
    }

    #[rustfmt::skip]
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::unnecessary_wraps)]
    fn loc(pair: Pair<Rule>) -> Result<Location, Error> {
        // Extract the register rule
        Ok(match pair.as_rule() {
            Rule::Byte => {
                let reg = pair.into_inner().next().ok_or(Error::ExpectedRule)?;
                Location::Byte(match reg.as_rule() {
                    Rule::A => cpu::reg::Byte::A,
                    Rule::F => cpu::reg::Byte::F,
                    Rule::B => cpu::reg::Byte::B,
                    Rule::C => cpu::reg::Byte::C,
                    Rule::D => cpu::reg::Byte::D,
                    Rule::E => cpu::reg::Byte::E,
                    Rule::H => cpu::reg::Byte::H,
                    Rule::L => cpu::reg::Byte::L,
                    rule => unreachable!("invalid rule: {rule:?}"),
                })
            }
            Rule::Word => {
                let reg = pair.into_inner().next().ok_or(Error::ExpectedRule)?;
                Location::Word(match reg.as_rule() {
                    Rule::AF   => cpu::reg::Word::AF,
                    Rule::BC   => cpu::reg::Word::BC,
                    Rule::DE   => cpu::reg::Word::DE,
                    Rule::HL   => cpu::reg::Word::HL,
                    Rule::SP   => cpu::reg::Word::SP,
                    Rule::PC   => cpu::reg::Word::PC,
                    rule => unreachable!("invalid rule: {rule:?}"),
                })
            }
            Rule::Pic => {
                let reg = pair.into_inner().next().ok_or(Error::ExpectedRule)?;
                Location::Pic(match reg.as_rule() {
                    Rule::If => pic::Control::If,
                    Rule::Ie => pic::Control::Ie,
                    rule => unreachable!("invalid rule: {rule:?}"),
                })
            }
            Rule::Ppu => {
                let reg = pair.into_inner().next().ok_or(Error::ExpectedRule)?;
                Location::Ppu(match reg.as_rule() {
                    Rule::Lcdc => ppu::Control::Lcdc,
                    Rule::Stat => ppu::Control::Stat,
                    Rule::Scy  => ppu::Control::Scy,
                    Rule::Scx  => ppu::Control::Scx,
                    Rule::Ly   => ppu::Control::Ly,
                    Rule::Lyc  => ppu::Control::Lyc,
                    Rule::Dma  => ppu::Control::Dma,
                    Rule::Bgp  => ppu::Control::Bgp,
                    Rule::Obp0 => ppu::Control::Obp0,
                    Rule::Obp1 => ppu::Control::Obp1,
                    Rule::Wy   => ppu::Control::Wy,
                    Rule::Wx   => ppu::Control::Wx,
                    rule => unreachable!("invalid rule: {rule:?}"),
                })
            }
            Rule::SerialX => {
                let reg = pair.into_inner().next().ok_or(Error::ExpectedRule)?;
                Location::Serial(match reg.as_rule() {
                    Rule::Sb => serial::Control::Sb,
                    Rule::Sc => serial::Control::Sc,
                    rule => unreachable!("invalid rule: {rule:?}"),
                })
            }
            Rule::Timer => {
                let reg = pair.into_inner().next().ok_or(Error::ExpectedRule)?;
                Location::Timer(match reg.as_rule() {
                    Rule::Div  => timer::Control::Div,
                    Rule::Tima => timer::Control::Tima,
                    Rule::Tma  => timer::Control::Tma,
                    Rule::Tac  => timer::Control::Tac,
                    rule => unreachable!("invalid rule: {rule:?}"),
                })
            }
            rule => unreachable!("invalid rule: {rule:?}"),
        })
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
    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("expected rule")]
    ExpectedRule,
}
