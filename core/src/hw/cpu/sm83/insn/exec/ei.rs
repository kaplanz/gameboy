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
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
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

fn execute(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0xfb {
        return Err(Error::Opcode(code));
    }

    // Execute EI
    cpu.ime = Ime::WillEnable;

    // Finish
    Ok(None)
}
