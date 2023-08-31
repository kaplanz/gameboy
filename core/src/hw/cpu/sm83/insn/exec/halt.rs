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
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
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

fn execute(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x76 {
        return Err(Error::Opcode(code));
    }

    // Perform HALT bug
    if !cpu.ime.enabled() && cpu.pic.borrow().int().is_some() {
        cpu.halt_bug = true;
    } else {
        // Execute HALT
        cpu.run = Status::Halted;
    }

    // Finish
    Ok(None)
}
