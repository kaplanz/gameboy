use rugby_arch::Byte;

use super::{Cpu, Error, Execute, Operation, Return, Status};

pub const fn default() -> Operation {
    Operation::Halt(Halt::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Halt {
    #[default]
    Execute,
}

impl Execute for Halt {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Halt> for Operation {
    fn from(value: Halt) -> Self {
        Self::Halt(value)
    }
}

fn execute(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x76 {
        return Err(Error::Opcode(code));
    }

    // Perform HALT bug
    if !cpu.etc.ime.enabled() && cpu.int.pending() {
        cpu.etc.halt_bug = true;
        // Do not execute HALT
        return Ok(None);
    }

    // Execute HALT
    cpu.etc.run = Status::Halted;

    // Finish
    Ok(None)
}
