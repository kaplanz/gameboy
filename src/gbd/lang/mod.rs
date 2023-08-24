use std::collections::VecDeque;
use std::ops::{Deref, DerefMut, Range};
use std::str::FromStr;

use gameboy::dmg::{cpu, ppu};

use super::Cycle;

mod parse;

pub use self::parse::Error;

#[derive(Clone, Debug)]
pub struct Program(VecDeque<Command>);

impl Program {
    pub fn new(prog: impl Iterator<Item = Command>) -> Self {
        Self(prog.collect())
    }
}

impl Deref for Program {
    type Target = VecDeque<Command>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Program {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for Program {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::prog(s)
    }
}

impl IntoIterator for Program {
    type Item = Command;

    type IntoIter = <VecDeque<Command> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    Break(u16),
    Continue,
    Delete(usize),
    Disable(usize),
    Enable(usize),
    Freq(Cycle),
    Help(Option<Keyword>),
    Ignore(usize, usize),
    Info(Option<Keyword>),
    Jump(u16),
    List,
    Load(Location),
    Log(Option<String>),
    Quit,
    Read(u16),
    ReadRange(Range<u16>),
    Reset,
    Step(Option<usize>),
    Store(Location, Value),
    Write(u16, u8),
    WriteRange(Range<u16>, u8),
}

#[derive(Clone, Debug)]
pub enum Keyword {
    Break,
    Continue,
    Delete,
    Disable,
    Enable,
    Freq,
    Help,
    Ignore,
    Info,
    Jump,
    List,
    Load,
    Log,
    Quit,
    Read,
    Reset,
    Step,
    Store,
    Write,
}

#[derive(Clone, Debug)]
pub enum Location {
    Byte(cpu::reg::Byte),
    Word(cpu::reg::Word),
    Control(ppu::Control),
}

#[derive(Clone, Debug)]
pub enum Value {
    Byte(u8),
    Word(u16),
}
