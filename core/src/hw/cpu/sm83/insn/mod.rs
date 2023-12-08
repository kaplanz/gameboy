//! Instruction state machine.

use std::fmt::{Debug, Display};

use log::trace;
use thiserror::Error;

use self::exec::Operation;
use super::{Cpu, Flag, Ime, Status};
use crate::hw::pic::Interrupt;

mod exec;
mod table;

type Result<T> = std::result::Result<T, Error>;

/// Instruction operation execution interface.
trait Execute {
    fn exec(self, opcode: u8, cpu: &mut Cpu) -> Result<Option<Operation>>;
}

/// CPU instruction state.
///
/// Stores the currently executing instruction together with its state. Can be
/// started and resumed.
#[derive(Clone, Debug)]
pub struct Instruction {
    code: u8,
    fmt: &'static str,
    oper: Operation,
}

impl Instruction {
    /// Constructs a new `Instruction` with the given opcode.
    #[must_use]
    pub fn new(opcode: u8) -> Self {
        table::DECODE[opcode as usize].clone()
    }

    /// Constructs a new prefix `Instruction` with the given opcode.
    #[must_use]
    pub fn prefix(opcode: u8) -> Self {
        table::PREFIX[opcode as usize].clone()
    }

    /// Constructs a new interrupt `Instruction`.
    pub(crate) fn int(int: Interrupt) -> Self {
        Self {
            code: int as u8,
            oper: Operation::Int(exec::int::Int::default()),
            fmt: int.repr(),
        }
    }

    /// Gets the instruction's opcode.
    #[must_use]
    pub fn opcode(&self) -> u8 {
        self.code
    }

    /// Executes a single stage of the instruction.
    ///
    /// # Errors
    ///
    /// Errors if the instruction failed to execute.
    pub fn exec(mut self, cpu: &mut Cpu) -> Result<Option<Self>> {
        // Execute operation
        trace!("{self:?}");
        let res = self.oper.exec(self.code, cpu)?;
        // Extract next stage
        self.oper = match res {
            Some(exec) => exec,
            None => return Ok(None),
        };
        // Return updated state
        Ok(Some(self))
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.fmt)
    }
}

/// Helper functions.
mod help {
    use remus::Cell;

    use super::Cpu;

    /// Get an 8-bit register operand.
    pub fn get_op8(cpu: &mut Cpu, idx: u8) -> u8 {
        match idx {
            0x0 => cpu.file.b.load(),
            0x1 => cpu.file.c.load(),
            0x2 => cpu.file.d.load(),
            0x3 => cpu.file.e.load(),
            0x4 => cpu.file.h.load(),
            0x5 => cpu.file.l.load(),
            0x6 => cpu.readbyte(),
            0x7 => cpu.file.a.load(),
            _ => panic!("Illegal register."),
        }
    }

    /// Set an 8-bit register operand.
    pub fn set_op8(cpu: &mut Cpu, idx: u8, value: u8) {
        match idx {
            0x0 => cpu.file.b.store(value),
            0x1 => cpu.file.c.store(value),
            0x2 => cpu.file.d.store(value),
            0x3 => cpu.file.e.store(value),
            0x4 => cpu.file.h.store(value),
            0x5 => cpu.file.l.store(value),
            0x6 => cpu.writebyte(value),
            0x7 => cpu.file.a.store(value),
            _ => panic!("Illegal register."),
        };
    }
}

/// A type specifying general categories of [`Instruction`] error.
#[derive(Debug, Error)]
pub enum Error {
    #[error("illegal instruction: {0:#04X}")]
    Illegal(u8),
    #[error("unexpected opcode: {0:#04X}")]
    Opcode(u8),
    #[error("unimplemented: {0:#04X}")]
    Unimplemented(u8),
    #[error("instruction overwrite: {0:?}")]
    Overwrite(Instruction),
}

#[cfg(test)]
mod tests;
