use remus::reg::Register;
use remus::Byte;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Daa(Daa::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Daa {
    #[default]
    Execute,
}

impl Execute for Daa {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Daa> for Operation {
    fn from(value: Daa) -> Self {
        Self::Daa(value)
    }
}

fn execute(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x27 {
        return Err(Error::Opcode(code));
    }

    // Execute DAA
    let flags = &cpu.reg.f.load();
    let didsub = Flag::N.get(flags);
    let hcarry = Flag::H.get(flags);
    let mut carry = Flag::C.get(flags);
    let mut adj = 0i8;
    let acc = cpu.reg.a.load();
    if hcarry || (!didsub && (acc & 0x0f) > 0x09) {
        adj |= 0x06;
    }
    if carry || (!didsub && acc > 0x99) {
        adj |= 0x60;
        carry = true;
    }
    adj = if didsub { -adj } else { adj };
    let res = (acc as i8).wrapping_add(adj) as Byte;
    cpu.reg.a.store(res);

    // Set flags
    let flags = &mut cpu.reg.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::H.set(flags, false);
    Flag::C.set(flags, carry);
    cpu.reg.f.store(*flags);

    // Finish
    Ok(None)
}
