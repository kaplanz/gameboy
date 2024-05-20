use remus::Byte;

use super::{Cpu, Error, Execute, Ime, Operation, Return};

pub const fn default() -> Operation {
    Operation::Ei(Ei::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Ei {
    #[default]
    Execute,
}

impl Execute for Ei {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Ei> for Operation {
    fn from(value: Ei) -> Self {
        Self::Ei(value)
    }
}

fn execute(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0xfb {
        return Err(Error::Opcode(code));
    }

    // Execute EI
    if matches!(cpu.etc.ime, Ime::Disabled) {
        cpu.etc.ime = Ime::WillEnable;
    }

    // Finish
    Ok(None)
}
