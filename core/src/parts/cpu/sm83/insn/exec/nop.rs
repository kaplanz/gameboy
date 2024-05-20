use remus::Byte;

use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Nop(Nop::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Nop {
    #[default]
    Execute,
}

impl Execute for Nop {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Nop> for Operation {
    fn from(value: Nop) -> Self {
        Self::Nop(value)
    }
}

fn execute(code: Byte, _cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x00 {
        return Err(Error::Opcode(code));
    }

    // Finish
    Ok(None)
}
