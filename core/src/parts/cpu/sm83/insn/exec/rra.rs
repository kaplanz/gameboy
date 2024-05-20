use enuf::Enuf;
use remus::reg::Register;
use remus::Byte;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Rra(Rra::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Rra {
    #[default]
    Execute,
}

impl Execute for Rra {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Rra> for Operation {
    fn from(value: Rra) -> Self {
        Self::Rra(value)
    }
}

fn execute(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x1f {
        return Err(Error::Opcode(code));
    }

    // Execute RRA
    let flags = &cpu.reg.f.load();
    let acc = cpu.reg.a.load();
    let cin = Flag::C.get(flags);
    let carry = acc & 0x01 != 0;
    let res = ((cin as Byte) << 7) | acc >> 1;
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
