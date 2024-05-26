//! Instruction set architecture.

use std::fmt::{Debug, Display};

use log::trace;
use rugby_arch::Byte;
use thiserror::Error;

use self::exec::Operation;
use super::{Cpu, Flag, Ime, Status};
use crate::parts::pic::Interrupt;

mod exec;
mod table;

/// Instruction operation execution interface.
trait Execute {
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Result<Option<Operation>>;
}

/// Processor instruction.
///
/// Stores the currently executing instruction together with its state. Can be
/// started and resumed.
#[derive(Clone, Debug)]
pub struct Instruction {
    code: Byte,
    repr: &'static str,
    oper: Operation,
}

impl Instruction {
    /// Constructs a new `Instruction` with the given opcode.
    #[must_use]
    pub fn decode(code: Byte) -> Self {
        table::DECODE[code as usize].clone()
    }

    /// Constructs a new prefix `Instruction` with the given opcode.
    #[must_use]
    pub fn prefix(code: Byte) -> Self {
        table::PREFIX[code as usize].clone()
    }

    /// Constructs a new interrupt `Instruction`.
    #[must_use]
    pub fn int(int: Interrupt) -> Self {
        Self {
            code: int as Byte,
            oper: Operation::Int(exec::int::Int::default()),
            repr: int.repr(),
        }
    }

    /// Gets the instruction's opcode.
    #[must_use]
    pub fn opcode(&self) -> Byte {
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
        write!(f, "{}", self.repr)
    }
}

impl From<Interrupt> for Instruction {
    fn from(value: Interrupt) -> Self {
        Self {
            code: value as Byte,
            oper: Operation::Int(exec::int::Int::default()),
            repr: value.repr(),
        }
    }
}

/// Helper functions.
mod help {
    use rugby_arch::reg::Register;
    use rugby_arch::Byte;

    use super::Cpu;

    /// Get an 8-bit register operand.
    pub fn get_op8(cpu: &mut Cpu, idx: Byte) -> Byte {
        match idx {
            0x0 => cpu.reg.b.load(),
            0x1 => cpu.reg.c.load(),
            0x2 => cpu.reg.d.load(),
            0x3 => cpu.reg.e.load(),
            0x4 => cpu.reg.h.load(),
            0x5 => cpu.reg.l.load(),
            0x6 => cpu.readbyte(),
            0x7 => cpu.reg.a.load(),
            _ => panic!("Illegal register."),
        }
    }

    /// Set an 8-bit register operand.
    pub fn set_op8(cpu: &mut Cpu, idx: Byte, value: Byte) {
        match idx {
            0x0 => cpu.reg.b.store(value),
            0x1 => cpu.reg.c.store(value),
            0x2 => cpu.reg.d.store(value),
            0x3 => cpu.reg.e.store(value),
            0x4 => cpu.reg.h.store(value),
            0x5 => cpu.reg.l.store(value),
            0x6 => cpu.writebyte(value),
            0x7 => cpu.reg.a.store(value),
            _ => panic!("Illegal register."),
        };
    }
}

/// A convenient type alias for [`Result`](std::result::Result).
pub type Result<T> = std::result::Result<T, Error>;

/// An error caused by an [instruction](Instruction).
#[derive(Debug, Error)]
pub enum Error {
    /// Illegal instruction.
    #[error("illegal instruction: {0:#04X}")]
    Illegal(Byte),
    /// Unexpected opcode.
    #[error("unexpected opcode: {0:#04X}")]
    Opcode(Byte),
    /// Unimplemented instruction.
    #[error("unimplemented: {0:#04X}")]
    Unimplemented(Byte),
}

#[cfg(test)]
mod tests;
