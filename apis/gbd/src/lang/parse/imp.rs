use std::num::ParseIntError;
use std::ops::RangeInclusive;
use std::panic;
use std::path::PathBuf;

use log::trace;
use num::traits::{WrappingAdd, WrappingSub};
use num::{Bounded, Integer};
use pest::iterators::Pair;
use rugby::core::dmg::{cpu, pic, ppu, serial, timer};
use thiserror::Error;
use wrange::Wrange;

use super::{Command, Keyword, Result, Rule, Select, Serial, Tick, Value};

#[allow(clippy::cast_sign_loss)]
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
        Rule::Capture => {
            let force = args
                .peek()
                .filter(|pair| pair.as_rule() == Rule::Force)
                .inspect(|_| {
                    args.next(); // consume only if found
                })
                .is_some();
            let path = args
                .next()
                .map(|pair| {
                    PathBuf::from(pair.as_span().as_str().to_string()).with_extension("png")
                })
                .exception()?;
            Command::Capture(path, force)
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
                        Rule::Dot   => Ok(Tick::Dot),
                        Rule::Mach  => Ok(Tick::Mach),
                        Rule::Insn  => Ok(Tick::Insn),
                        Rule::Line  => Ok(Tick::Line),
                        Rule::Frame => Ok(Tick::Frame),
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
            let pair = args.next().exception()?;
            let mode = match pair.as_rule() {
                Rule::Peek => Serial::Peek,
                Rule::Recv => Serial::Recv,
                Rule::Send => Serial::Send({
                    let data = pair.into_inner().next().exception()?;
                    match data.as_rule() {
                        Rule::Bytes => {
                            data
                                // extract inner
                                .into_inner()
                                // parse each byte
                                .map(self::integer)
                                // collect into a vec
                                .collect()
                        }
                        Rule::String => {
                            data
                                // extract inner
                                .into_inner()
                                // find only argument
                                .next()
                                .exception()
                                // parse as string
                                .map(|inner| inner.as_str().as_bytes().to_vec())
                        }
                        rule => rule.exception(),
                    }?
                }),
                rule => return rule.exception(),
            };
            Command::Serial(mode)
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
                Select::Byte(_) | Select::Pic(_) | Select::Ppu(_) | Select::Serial(_) | Select::Timer(_) => Value::Byte(
                    self::integer(value.clone()) // attempt both `u8` and `i8`
                        .or_else(|_| self::integer::<i8>(value).map(|int| int as u8))?,
                ),
                Select::Word(_) => Value::Word(
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
    let (radix, value) = {
        let mut num = int.next().exception()?.into_inner();
        let radix = num.next().exception()?;
        let value = num.next().exception()?;
        (radix, value)
    };
    let sign = int.next().map_or("", |rule| rule.as_str()).to_string();
    // Parse into an integer type
    match radix.as_rule() {
        Rule::BinRadix => I::from_str_radix(&(sign + value.as_str()), 2),
        Rule::OctRadix => I::from_str_radix(&(sign + value.as_str()), 8),
        Rule::DecRadix => I::from_str_radix(&(sign + value.as_str()), 10),
        Rule::HexRadix => I::from_str_radix(&(sign + value.as_str()), 16),
        rule => return Err(Error::from(ErrorKind::Invalid(rule)).into()),
    }
    .map_err(super::Error::ParseInt)
}

pub fn range<I>(pair: Pair<Rule>) -> Result<Wrange<I>>
where
    I: Bounded
        + Clone
        + Copy
        + Integer<FromStrRadixErr = ParseIntError>
        + WrappingAdd
        + WrappingSub
        + 'static,
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
            let stx: I = self::integer(stx)?;
            let end: I = match end.as_rule() {
                Rule::UInt => self::integer(end)?,
                Rule::SInt => stx.wrapping_add(&self::integer(end)?),
                rule => return Err(Error::from(ErrorKind::Invalid(rule)).into()),
            };
            // Define
            let range = Wrange::from(stx..end);
            Ok(range)
        }
        Rule::RangeFrom => {
            let mut range = pair.into_inner();
            // Extract
            let stx = range.next().exception()?;
            // Parse
            let stx = self::integer(stx)?;
            // Define
            let range = Wrange::from(stx..);
            Ok(range)
        }
        Rule::RangeFull => {
            // Define
            let range = Wrange::from(..);
            Ok(range)
        }
        Rule::RangeInc => {
            let mut range = pair.into_inner();
            // Extract
            let stx = range.next().exception()?;
            let end = range.next().exception()?;
            // Parse
            let stx: I = self::integer(stx)?;
            let end: I = match end.as_rule() {
                Rule::UInt => self::integer(end)?,
                Rule::SInt => stx.wrapping_add(&self::integer(end)?),
                rule => return Err(Error::from(ErrorKind::Invalid(rule)).into()),
            };
            // Define
            let range = Wrange::from(stx..=end);
            Ok(range)
        }
        Rule::RangeTo => {
            let mut range = pair.into_inner();
            // Extract
            let end = range.next().exception()?;
            // Parse
            let end: I = match end.as_rule() {
                Rule::UInt => self::integer(end)?,
                Rule::SInt => I::zero().wrapping_add(&self::integer(end)?),
                rule => return Err(Error::from(ErrorKind::Invalid(rule)).into()),
            };
            // Define
            let range = Wrange::from(..end);
            Ok(range)
        }
        Rule::RangeToInc => {
            let mut range = pair.into_inner();
            // Extract
            let end = range.next().exception()?;
            // Parse
            let end: I = match end.as_rule() {
                Rule::UInt => self::integer(end)?,
                Rule::SInt => I::zero().wrapping_add(&self::integer(end)?),
                rule => return Err(Error::from(ErrorKind::Invalid(rule)).into()),
            };
            // Define
            let range = Wrange::from(..=end);
            Ok(range)
        }
        rule => rule.exception(),
    }
}

#[rustfmt::skip]
#[allow(clippy::needless_pass_by_value)]
pub fn keyword(pair: Pair<Rule>) -> Result<Keyword> {
    // Extract the keyword rule
    Ok(match pair.as_rule() {
        Rule::KBreak    => Keyword::Break,
        Rule::KCapture  => Keyword::Capture,
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
pub fn location(pair: Pair<Rule>) -> Result<Select> {
    // Extract the register rule
    Ok(match pair.as_rule() {
        Rule::Byte => {
            let reg = pair.into_inner().next().exception()?;
            Select::Byte(match reg.as_rule() {
                Rule::A => cpu::Select8::A,
                Rule::F => cpu::Select8::F,
                Rule::B => cpu::Select8::B,
                Rule::C => cpu::Select8::C,
                Rule::D => cpu::Select8::D,
                Rule::E => cpu::Select8::E,
                Rule::H => cpu::Select8::H,
                Rule::L => cpu::Select8::L,
                rule => return rule.exception(),
            })
        }
        Rule::Word => {
            let reg = pair.into_inner().next().exception()?;
            Select::Word(match reg.as_rule() {
                Rule::AF => cpu::Select16::AF,
                Rule::BC => cpu::Select16::BC,
                Rule::DE => cpu::Select16::DE,
                Rule::HL => cpu::Select16::HL,
                Rule::SP => cpu::Select16::SP,
                Rule::PC => cpu::Select16::PC,
                rule => return rule.exception(),
            })
        }
        Rule::Pic => {
            let reg = pair.into_inner().next().exception()?;
            Select::Pic(match reg.as_rule() {
                Rule::If => pic::Select::If,
                Rule::Ie => pic::Select::Ie,
                rule => return rule.exception(),
            })
        }
        Rule::Ppu => {
            let reg = pair.into_inner().next().exception()?;
            Select::Ppu(match reg.as_rule() {
                Rule::Lcdc => ppu::Select::Lcdc,
                Rule::Stat => ppu::Select::Stat,
                Rule::Scy  => ppu::Select::Scy,
                Rule::Scx  => ppu::Select::Scx,
                Rule::Ly   => ppu::Select::Ly,
                Rule::Lyc  => ppu::Select::Lyc,
                Rule::Dma  => ppu::Select::Dma,
                Rule::Bgp  => ppu::Select::Bgp,
                Rule::Obp0 => ppu::Select::Obp0,
                Rule::Obp1 => ppu::Select::Obp1,
                Rule::Wy   => ppu::Select::Wy,
                Rule::Wx   => ppu::Select::Wx,
                rule => return rule.exception(),
            })
        }
        Rule::SerialX => {
            let reg = pair.into_inner().next().exception()?;
            Select::Serial(match reg.as_rule() {
                Rule::Sb => serial::Select::Sb,
                Rule::Sc => serial::Select::Sc,
                rule => return rule.exception(),
            })
        }
        Rule::Timer => {
            let reg = pair.into_inner().next().exception()?;
            Select::Timer(match reg.as_rule() {
                Rule::Div  => timer::Select::Div,
                Rule::Tima => timer::Select::Tima,
                Rule::Tma  => timer::Select::Tma,
                Rule::Tac  => timer::Select::Tac,
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
    /// Expected pair, found none.
    #[error("expected pair, found none")]
    Expected,
    /// Invalid rule.
    #[error("invalid rule: {0:?}")]
    Invalid(Rule),
}

/// Exception shorthand for parsing errors.
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
