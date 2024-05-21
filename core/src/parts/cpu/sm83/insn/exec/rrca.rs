use remus::reg::Register;
use remus::Byte;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Rrca(Rrca::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Rrca {
    #[default]
    Execute,
}

impl Execute for Rrca {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Rrca> for Operation {
    fn from(value: Rrca) -> Self {
        Self::Rrca(value)
    }
}

fn execute(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x0f {
        return Err(Error::Opcode(code));
    }

    // Execute RRCA
    let acc = cpu.reg.a.load();
    let carry = acc & 0x01 != 0;
    let res = ((carry as Byte) << 7) | acc >> 1;
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
