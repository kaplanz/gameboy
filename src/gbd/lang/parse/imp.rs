use std::num::ParseIntError;
use std::ops::RangeInclusive;
use std::panic;

use derange::Derange;
use log::trace;
use num::traits::WrappingSub;
use num::{Bounded, Integer};
use pest::iterators::Pair;
use thiserror::Error;

use super::{Command, Freq, Keyword, Location, Result, Rule, Value};
use crate::core::dmg::{cpu, pic, ppu, serial, timer};

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::similar_names)]
#[allow(clippy::too_many_lines)]
pub fn command(input: Pair<Rule>) -> Result<Command> {
    // Extract keyword and args
    let rule = input.as_rule();
    let mut args = input.into_inner();
    let kword = args.next().exception()?;
    trace!(
        "keyword: {kword:?}, args: {args:?}",
        kword = kword.as_rule(),
        args = args.clone().map(|arg| arg.as_rule()).collect::<Vec<_>>()
    );

    // Parse individual command
    let cmd = match rule {
        Rule::Break => {
            let addr = args.next().exception().and_then(self::integer)?;
            Command::Break(addr)
        }
        Rule::Continue => Command::Continue,
        Rule::Delete => {
            let index = args.next().exception().and_then(self::integer)?;
            Command::Delete(index)
        }
        Rule::Disable => {
            let index = args.next().exception().and_then(self::integer)?;
            Command::Disable(index)
        }
        Rule::Enable => {
            let index = args.next().exception().and_then(self::integer)?;
            Command::Enable(index)
        }
        Rule::Freq => {
            #[rustfmt::skip]
            let mode = args
                .next()
                .map(|pair| -> Result<_> {
                    match pair.as_rule() {
                        Rule::Dot   => Ok(Freq::Dot),
                        Rule::Mach  => Ok(Freq::Mach),
                        Rule::Insn  => Ok(Freq::Insn),
                        Rule::Line  => Ok(Freq::Line),
                        Rule::Frame => Ok(Freq::Frame),
                        rule => rule.exception(),
                    }
                })
                .transpose()?;
            Command::Freq(mode)
        }
        Rule::Goto => {
            let addr = args.next().exception().and_then(self::integer)?;
            Command::Goto(addr)
        }
        Rule::Help => {
            let what = args.next().map(self::keyword).transpose()?;
            Command::Help(what)
        }
        Rule::Ignore => {
            let index = args.next().exception().and_then(self::integer)?;
            let count = args.next().exception().and_then(self::integer)?;
            Command::Ignore(index, count)
        }
        Rule::Info => {
            let what = args.next().map(self::keyword).transpose()?;
            Command::Info(what)
        }
        Rule::Jump => {
            let addr = args.next().exception().and_then(self::integer)?;
            Command::Jump(addr)
        }
        Rule::List => Command::List,
        Rule::Load => {
            let loc = args.map(self::location).collect::<Result<_>>()?;
            Command::Load(loc)
        }
        Rule::Log => {
            let filter = args.next().map(|pair| pair.as_span().as_str().to_string());
            Command::Log(filter)
        }
        Rule::Quit => Command::Quit,
        Rule::Read => {
            let what = args.next().exception()?;
            // Match on address (range)
            match what.as_rule() {
                Rule::UInt => {
                    let addr = self::integer(what)?;
                    Command::Read(addr)
                }
                Rule::RangeBounds => {
                    let mut pairs = what.into_inner();
                    // Match on range bounds
                    let pair = pairs.next().exception()?;
                    let range = self::range(pair)?;
                    Command::ReadRange(range)
                }
                rule => return rule.exception(),
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
                                .map(self::integer)
                                // collect into a vec
                                .collect()
                        }
                        Rule::String => {
                            pair
                                // extract inner
                                .into_inner()
                                // find only argument
                                .next()
                                .exception()
                                // parse as string
                                .map(|inner| inner.as_str().as_bytes().to_vec())
                        }
                        rule => rule.exception(),
                    }
                })
                .transpose()?;
            Command::Serial(data)
        }
        Rule::Step => {
            let many = args.next().map(self::integer).transpose()?;
            Command::Step(many)
        }
        Rule::Store => {
            let mut args = args.rev(); // must extract the value first
            let value = args.next().exception()?;
            let locs = args
                .rev() // undo previous reverse to perform stores in order
                .map(self::location)
                .collect::<Result<Vec<_>>>()?;
            let value = match locs.first().exception()? {
                Location::Byte(_) | Location::Pic(_) | Location::Ppu(_) | Location::Serial(_) | Location::Timer(_) => Value::Byte(
                    self::integer(value.clone()) // attempt both `u8` and `i8`
                        .or_else(|_| self::integer::<i8>(value).map(|int| int as u8))?,
                ),
                Location::Word(_) => Value::Word(
                    self::integer(value.clone()) // attempt both `u16` and `i16`
                        .or_else(|_| self::integer::<i16>(value).map(|int| int as u16))?,
                ),
            };
            Command::Store(locs, value)
        }
        Rule::Write => {
            let what = args.next().exception()?;
            // Match on data byte
            let pair = args.next().exception()?;
            let byte =
                self::integer(pair.clone()) // attempt both `u8` and `i8`
                    .or_else(|_| self::integer::<i8>(pair).map(|int| int as u8))?;
            // Match on address (range)
            match what.as_rule() {
                Rule::UInt => {
                    let addr = self::integer(what)?;
                    Command::Write(addr, byte)
                }
                Rule::RangeBounds => {
                    let mut pairs = what.into_inner();
                    // Match on range bounds
                    let pair = pairs.next().exception()?;
                    let range = self::range(pair)?;
                    Command::WriteRange(range, byte)
                }
                rule => return rule.exception(),
            }
        }
        rule => return rule.exception(),
    };

    Ok(cmd)
}

pub fn integer<I>(pair: Pair<Rule>) -> Result<I>
where
    I: Integer<FromStrRadixErr = ParseIntError>,
{
    // Extract the number and sign
    let mut int = pair
        .into_inner() // `Int` is composed of `Sign` and `Num`
        .rev(); // since sign is optional, reverse to process it last
    let num = int.next().exception()?;
    let sign = int.next().map_or("", |rule| rule.as_str()).to_string();
    // Parse into an integer type
    match num.as_rule() {
        Rule::Bin => I::from_str_radix(&(sign + &num.as_str()[2..]), 2),
        Rule::Oct => I::from_str_radix(&(sign + &num.as_str()[2..]), 8),
        Rule::Dec => I::from_str_radix(&(sign + &num.as_str()[0..]), 10),
        Rule::Hex => I::from_str_radix(&(sign + &num.as_str()[2..]), 16),
        rule => return Err(Error::from(ErrorKind::Invalid(rule)).into()),
    }
    .map_err(super::Error::ParseInt)
}

pub fn range<I>(pair: Pair<Rule>) -> Result<Derange<I>>
where
    I: Bounded + Clone + Copy + Integer<FromStrRadixErr = ParseIntError> + WrappingSub + 'static,
    RangeInclusive<I>: Iterator<Item = I>,
{
    // Extract the range rule
    match pair.as_rule() {
        Rule::Range => {
            let mut range = pair.into_inner();
            // Extract
            let stx = range.next().exception()?;
            let end = range.next().exception()?;
            // Parse
            let stx = self::integer(stx)?;
            let end = self::integer(end)?;
            // Define
            let range = Derange::from(stx..end);
            Ok(range)
        }
        Rule::RangeFrom => {
            let mut range = pair.into_inner();
            // Extract
            let stx = range.next().exception()?;
            // Parse
            let stx = self::integer(stx)?;
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
            let stx = range.next().exception()?;
            let end = range.next().exception()?;
            // Parse
            let stx = self::integer(stx)?;
            let end = self::integer(end)?;
            // Define
            let range = Derange::from(stx..=end);
            Ok(range)
        }
        Rule::RangeTo => {
            let mut range = pair.into_inner();
            // Extract
            let end = range.next().exception()?;
            // Parse
            let end = self::integer(end)?;
            // Define
            let range = Derange::from(..end);
            Ok(range)
        }
        Rule::RangeToInc => {
            let mut range = pair.into_inner();
            // Extract
            let end = range.next().exception()?;
            // Parse
            let end = self::integer(end)?;
            // Define
            let range = Derange::from(..=end);
            Ok(range)
        }
        rule => rule.exception(),
    }
}

#[rustfmt::skip]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn keyword(pair: Pair<Rule>) -> Result<Keyword> {
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
        rule => return rule.exception(),
    })
}

#[rustfmt::skip]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn location(pair: Pair<Rule>) -> Result<Location> {
    // Extract the register rule
    Ok(match pair.as_rule() {
        Rule::Byte => {
            let reg = pair.into_inner().next().exception()?;
            Location::Byte(match reg.as_rule() {
                Rule::A => cpu::reg::Byte::A,
                Rule::F => cpu::reg::Byte::F,
                Rule::B => cpu::reg::Byte::B,
                Rule::C => cpu::reg::Byte::C,
                Rule::D => cpu::reg::Byte::D,
                Rule::E => cpu::reg::Byte::E,
                Rule::H => cpu::reg::Byte::H,
                Rule::L => cpu::reg::Byte::L,
                rule => return rule.exception(),
            })
        }
        Rule::Word => {
            let reg = pair.into_inner().next().exception()?;
            Location::Word(match reg.as_rule() {
                Rule::AF => cpu::reg::Word::AF,
                Rule::BC => cpu::reg::Word::BC,
                Rule::DE => cpu::reg::Word::DE,
                Rule::HL => cpu::reg::Word::HL,
                Rule::SP => cpu::reg::Word::SP,
                Rule::PC => cpu::reg::Word::PC,
                rule => return rule.exception(),
            })
        }
        Rule::Pic => {
            let reg = pair.into_inner().next().exception()?;
            Location::Pic(match reg.as_rule() {
                Rule::If => pic::Control::If,
                Rule::Ie => pic::Control::Ie,
                rule => return rule.exception(),
            })
        }
        Rule::Ppu => {
            let reg = pair.into_inner().next().exception()?;
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
                rule => return rule.exception(),
            })
        }
        Rule::SerialX => {
            let reg = pair.into_inner().next().exception()?;
            Location::Serial(match reg.as_rule() {
                Rule::Sb => serial::Control::Sb,
                Rule::Sc => serial::Control::Sc,
                rule => return rule.exception(),
            })
        }
        Rule::Timer => {
            let reg = pair.into_inner().next().exception()?;
            Location::Timer(match reg.as_rule() {
                Rule::Div  => timer::Control::Div,
                Rule::Tima => timer::Control::Tima,
                Rule::Tma  => timer::Control::Tma,
                Rule::Tac  => timer::Control::Tac,
                rule => return rule.exception(),
            })
        }
        rule => return rule.exception(),
    })
}

/// An internal error which can be returned when parsing.
///
/// # Note
///
/// Internal errors are always considered bugs, and should never happen due to
/// poor user input.
#[derive(Debug, Error)]
#[error("{kind}: {location}")]
pub struct Error {
    kind: ErrorKind,
    location: &'static panic::Location<'static>,
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self {
            kind,
            location: panic::Location::caller(),
        }
    }
}

/// A type specifying categories of parser errors.
#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("expected pair, found none")]
    Expected,
    #[error("invalid rule: {0:?}")]
    Invalid(Rule),
}

/// Exception shorthand for [`Language`] errors.
trait Exception<T> {
    type Err: std::error::Error;

    /// Transforms `Self` into a [`Result<T, Self::Error>`].
    fn exception(self) -> Result<T, Self::Err>;
}

impl<T> Exception<T> for Option<T> {
    type Err = super::Error;

    /// Transforms the [`Option<T>`] into a [`Result<T, E>`], mapping
    /// [`Some(v)`] to [`Ok(v)`] and [`None`] to [`Err(Error::Expected)`].
    #[track_caller]
    fn exception(self) -> Result<T, Self::Err> {
        self.ok_or(
            Error {
                kind: ErrorKind::Expected,
                location: panic::Location::caller(),
            }
            .into(),
        )
    }
}

impl<T> Exception<T> for Rule {
    type Err = super::Error;

    /// Transforms the [`Option<T>`] into a [`Result<T, E>`], mapping
    /// [`Some(v)`] to [`Ok(v)`] and [`None`] to [`Err(Error::Expected)`].
    #[track_caller]
    fn exception(self) -> Result<T, Self::Err> {
        Err(Error {
            kind: ErrorKind::Invalid(self),
            location: panic::Location::caller(),
        }
        .into())
    }
}
