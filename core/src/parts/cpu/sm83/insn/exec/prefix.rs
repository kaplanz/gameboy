use remus::Byte;

use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Prefix(Prefix::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Prefix {
    #[default]
    Fetch,
}

impl Execute for Prefix {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
        }
    }
}

impl From<Prefix> for Operation {
    fn from(value: Prefix) -> Self {
        Self::Prefix(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0xcb {
        return Err(Error::Opcode(code));
    }

    // Execute PREFIX
    cpu.etc.prefix = true;

    // Proceed
    Ok(None)
}
