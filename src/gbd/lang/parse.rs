use std::fmt::Display;
use std::num::ParseIntError;
use std::ops::RangeInclusive;
use std::panic;

use derange::Derange;
use log::trace;
use num::traits::WrappingSub;
use num::{Bounded, Integer};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use super::{Command, Freq, Keyword, Location, Program, Value};
use crate::core::dmg::{cpu, pic, ppu, serial, timer};

type Result<T, E = Error> = std::result::Result<T, E>;

pub fn prog(src: &str) -> Result<Program> {
    Language::prog(src)
        .map(IntoIterator::into_iter)
        .map(Program::new)
}

#[derive(Debug, Parser)]
#[grammar = "gbd/lang/parse.pest"]
struct Language;

impl Language {
    fn prog(src: &str) -> Result<Vec<Command>> {
        // Parse program string
        let pairs = Language::parse(Rule::Program, src)
            .map_err(|err| err.renamed_rules(ToString::to_string))?;
        // Extract individual commands
        pairs
            .filter(|pair| !matches!(pair.as_rule(), Rule::EOI))
            .map(Self::command)
            .collect()
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::similar_names)]
    #[allow(clippy::too_many_lines)]
    fn command(input: Pair<Rule>) -> Result<Command> {
        // Extract keyword and args
        let rule = input.as_rule();
        let mut args = input.into_inner();
        let kword = args.next().except()?;
        trace!(
            "keyword: {kword:?}, args: {args:?}",
            kword = kword.as_rule(),
            args = args.clone().map(|arg| arg.as_rule()).collect::<Vec<_>>()
        );

        // Parse individual command
        let cmd = match rule {
            Rule::Break => {
                let addr = args.next().except().and_then(Self::integer)?;
                Command::Break(addr)
            }
            Rule::Continue => Command::Continue,
            Rule::Delete => {
                let index = args.next().except().and_then(Self::integer)?;
                Command::Delete(index)
            }
            Rule::Disable => {
                let index = args.next().except().and_then(Self::integer)?;
                Command::Disable(index)
            }
            Rule::Enable => {
                let index = args.next().except().and_then(Self::integer)?;
                Command::Enable(index)
            }
            Rule::Freq => {
                #[rustfmt::skip]
                let mode = args.next().map(|pair| match pair.as_rule() {
                    Rule::Dot   => Freq::Dot,
                    Rule::Mach  => Freq::Mach,
                    Rule::Insn  => Freq::Insn,
                    Rule::Line  => Freq::Line,
                    Rule::Frame => Freq::Frame,
                    rule => unreachable!("invalid rule: {rule:?}"),
                });
                Command::Freq(mode)
            }
            Rule::Goto => {
                let addr = args.next().except().and_then(Self::integer)?;
                Command::Goto(addr)
            }
            Rule::Help => {
                let what = args.next().map(Self::keyword).transpose()?;
                Command::Help(what)
            }
            Rule::Ignore => {
                let index = args.next().except().and_then(Self::integer)?;
                let count = args.next().except().and_then(Self::integer)?;
                Command::Ignore(index, count)
            }
            Rule::Info => {
                let what = args.next().map(Self::keyword).transpose()?;
                Command::Info(what)
            }
            Rule::Jump => {
                let addr = args.next().except().and_then(Self::integer)?;
                Command::Jump(addr)
            }
            Rule::List => Command::List,
            Rule::Load => {
                let loc = args.map(Self::location).collect::<Result<_, _>>()?;
                Command::Load(loc)
            }
            Rule::Log => {
                let filter = args.next().map(|pair| pair.as_span().as_str().to_string());
                Command::Log(filter)
            }
            Rule::Quit => Command::Quit,
            Rule::Read => {
                let what = args.next().except()?;
                // Match on address (range)
                match what.as_rule() {
                    Rule::UInt => {
                        let addr = Self::integer(what)?;
                        Command::Read(addr)
                    }
                    Rule::RangeBounds => {
                        let mut pairs = what.into_inner();
                        // Match on range bounds
                        let pair = pairs.next().except()?;
                        let range = Self::range(pair)?;
                        Command::ReadRange(range)
                    }
                    rule => unreachable!("invalid rule: {rule:?}"),
                }
            }
            Rule::Reset => Command::Reset,
            Rule::Serial => {
                let data = args
                    .next()
                    .map(|pair| -> Result<_> {
                        match pair.as_rule() {
                            Rule::Bytes => {
                                pair
                                    // extract inner
                                    .into_inner()
                                    // parse each byte
                                    .map(Self::integer)
                                    // collect into a vec
                                    .collect()
                            }
                            Rule::String => {
                                pair
                                    // extract inner
                                    .into_inner()
                                    // find only argument
                                    .next()
                                    .except()
                                    // parse as string
                                    .map(|inner| inner.as_str().as_bytes().to_vec())
                            }
                            rule => unreachable!("invalid rule: {rule:?}"),
                        }
                    })
                    .transpose()?;
                Command::Serial(data)
            }
            Rule::Step => {
                let many = args.next().map(Self::integer).transpose()?;
                Command::Step(many)
            }
            Rule::Store => {
                let mut args = args.rev(); // must extract the value first
                let value = args.next().except()?;
                let locs = args
                    .rev() // undo previous reverse to perform stores in order
                    .map(Self::location)
                    .collect::<Result<Vec<_>, _>>()?;
                let value = match locs.first().except()? {
                    Location::Byte(_) | Location::Pic(_) | Location::Ppu(_) | Location::Serial(_) | Location::Timer(_) => Value::Byte(
                        Self::integer(value.clone()) // attempt both `u8` and `i8`
                            .or_else(|_| Self::integer::<i8>(value).map(|int| int as u8))?,
                    ),
                    Location::Word(_) => Value::Word(
                        Self::integer(value.clone()) // attempt both `u16` and `i16`
                            .or_else(|_| Self::integer::<i16>(value).map(|int| int as u16))?,
                    ),
                };
                Command::Store(locs, value)
            }
            Rule::Write => {
                let what = args.next().except()?;
                // Match on data byte
                let pair = args.next().except()?;
                let byte = Self::integer(pair.clone()) // attempt both `u8` and `i8`
                    .or_else(|_| Self::integer::<i8>(pair).map(|int| int as u8))?;
                // Match on address (range)
                match what.as_rule() {
                    Rule::UInt => {
                        let addr = Self::integer(what)?;
                        Command::Write(addr, byte)
                    }
                    Rule::RangeBounds => {
                        let mut pairs = what.into_inner();
                        // Match on range bounds
                        let pair = pairs.next().except()?;
                        let range = Self::range(pair)?;
                        Command::WriteRange(range, byte)
                    }
                    rule => unreachable!("invalid rule: {rule:?}"),
                }
            }
            rule => unreachable!("invalid rule: {rule:?}"),
        };

        Ok(cmd)
    }

    fn integer<I>(pair: Pair<Rule>) -> Result<I>
    where
        I: Integer<FromStrRadixErr = ParseIntError>,
    {
        // Extract the number and sign
        let mut int = pair
            .into_inner() // `Int` is composed of `Sign` and `Num`
            .rev(); // since sign is optional, reverse to process it last
        let num = int.next().except()?;
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

    fn range<I>(pair: Pair<Rule>) -> Result<Derange<I>>
    where
        I: Bounded
            + Clone
            + Copy
            + Integer<FromStrRadixErr = ParseIntError>
            + WrappingSub
            + 'static,
        RangeInclusive<I>: Iterator<Item = I>,
    {
        // Extract the range rule
        match pair.as_rule() {
            Rule::Range => {
                let mut range = pair.into_inner();
                // Extract
                let stx = range.next().except()?;
                let end = range.next().except()?;
                // Parse
                let stx = Self::integer(stx)?;
                let end = Self::integer(end)?;
                // Define
                let range = Derange::from(stx..end);
                Ok(range)
            }
            Rule::RangeFrom => {
                let mut range = pair.into_inner();
                // Extract
                let stx = range.next().except()?;
                // Parse
                let stx = Self::integer(stx)?;
                // Define
                let range = Derange::from(stx..);
                Ok(range)
            }
            Rule::RangeFull => {
                // Define
                let range = Derange::from(..);
                Ok(range)
            }
            Rule::RangeInc => {
                let mut range = pair.into_inner();
                // Extract
                let stx = range.next().except()?;
                let end = range.next().except()?;
                // Parse
                let stx = Self::integer(stx)?;
                let end = Self::integer(end)?;
                // Define
                let range = Derange::from(stx..=end);
                Ok(range)
            }
            Rule::RangeTo => {
                let mut range = pair.into_inner();
                // Extract
                let end = range.next().except()?;
                // Parse
                let end = Self::integer(end)?;
                // Define
                let range = Derange::from(..end);
                Ok(range)
            }
            Rule::RangeToInc => {
                let mut range = pair.into_inner();
                // Extract
                let end = range.next().except()?;
                // Parse
                let end = Self::integer(end)?;
                // Define
                let range = Derange::from(..=end);
                Ok(range)
            }
            rule => unreachable!("invalid rule: {rule:?}"),
        }
    }

    #[rustfmt::skip]
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::unnecessary_wraps)]
    fn keyword(pair: Pair<Rule>) -> Result<Keyword> {
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
    fn location(pair: Pair<Rule>) -> Result<Location> {
        // Extract the register rule
        Ok(match pair.as_rule() {
            Rule::Byte => {
                let reg = pair.into_inner().next().except()?;
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
                let reg = pair.into_inner().next().except()?;
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
                let reg = pair.into_inner().next().except()?;
                Location::Pic(match reg.as_rule() {
                    Rule::If => pic::Control::If,
                    Rule::Ie => pic::Control::Ie,
                    rule => unreachable!("invalid rule: {rule:?}"),
                })
            }
            Rule::Ppu => {
                let reg = pair.into_inner().next().except()?;
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
                let reg = pair.into_inner().next().except()?;
                Location::Serial(match reg.as_rule() {
                    Rule::Sb => serial::Control::Sb,
                    Rule::Sc => serial::Control::Sc,
                    rule => unreachable!("invalid rule: {rule:?}"),
                })
            }
            Rule::Timer => {
                let reg = pair.into_inner().next().except()?;
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
    #[error("internal error: {0}")]
    Internal(&'static panic::Location<'static>),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),
}

/// Exception shorthand for [`Language`] errors.
trait Except<T> {
    type Error: std::error::Error;

    /// Transforms `Self` into a [`Result<T, Self::Error>`].
    fn except(self) -> Result<T, Self::Error>;
}

impl<T> Except<T> for Option<T> {
    type Error = Error;

    /// Transforms the [`Option<T>`] into a [`Result<T, E>`], mapping
    /// [`Some(v)`] to [`Ok(v)`] and [`None`] to [`Err(Error::Internal(_))`].
    #[track_caller]
    fn except(self) -> Result<T, Self::Error> {
        self.ok_or(Error::Internal(panic::Location::caller()))
    }
}
