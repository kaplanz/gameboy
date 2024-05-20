use remus::Byte;

use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Unused(Unused::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Unused {
    #[default]
    Execute,
}

impl Execute for Unused {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Unused> for Operation {
    fn from(value: Unused) -> Self {
        Self::Unused(value)
    }
}

fn execute(code: Byte, _: &mut Cpu) -> Return {
    Err(Error::Illegal(code))
}
