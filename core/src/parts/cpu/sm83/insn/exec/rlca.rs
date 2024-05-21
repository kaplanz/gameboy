use remus::reg::Register;
use remus::Byte;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Rlca(Rlca::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Rlca {
    #[default]
    Execute,
}

impl Execute for Rlca {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Rlca> for Operation {
    fn from(value: Rlca) -> Self {
        Self::Rlca(value)
    }
}

fn execute(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x07 {
        return Err(Error::Opcode(code));
    }

    // Execute RLCA
    let acc = cpu.reg.a.load();
    let carry = acc & 0x80 != 0;
    let res = acc << 1 | (carry as Byte);
    cpu.reg.a.store(res);

    // Set flags
    let flags = &mut cpu.reg.f.load();
    Flag::Z.set(flags, false);
    Flag::N.set(flags, false);
    Flag::H.set(flags, false);
    Flag::C.set(flags, carry);
    cpu.reg.f.store(*flags);

    // Finish
    Ok(None)
}
