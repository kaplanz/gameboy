use std::collections::VecDeque;
use std::ops::{Deref, DerefMut, Range};
use std::str::FromStr;

use gameboy::core::cpu::sm83::Register;

pub use self::parse::Error;
use super::Cycle;

mod parse;

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
    Freq(Cycle),
    Help(Option<String>),
    Info(Option<String>),
    Jump(u16),
    List,
    Load(Register),
    Log(Option<String>),
    Quit,
    Read(u16),
    ReadRange(Range<u16>),
    Reset,
    Skip(usize, usize),
    Step,
    Store(Register, u16),
    Write(u16, u8),
    WriteRange(Range<u16>, u8),
}
