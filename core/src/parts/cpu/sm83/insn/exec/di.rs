use remus::Byte;

use super::{Cpu, Error, Execute, Ime, Operation, Return};

pub const fn default() -> Operation {
    Operation::Di(Di::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Di {
    #[default]
    Execute,
}

impl Execute for Di {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Di> for Operation {
    fn from(value: Di) -> Self {
        Self::Di(value)
    }
}

fn execute(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0xf3 {
        return Err(Error::Opcode(code));
    }

    // Execute DI
    cpu.etc.ime = Ime::Disabled;

    // Finish
    Ok(None)
}
