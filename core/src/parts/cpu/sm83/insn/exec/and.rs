use std::ops::BitAnd;

use remus::reg::Register;
use remus::Byte;

use super::{help, Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::And(And::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum And {
    #[default]
    Fetch,
    Execute(Byte),
}

impl Execute for And {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op2) => execute(code, cpu, op2),
        }
    }
}

impl From<And> for Operation {
    fn from(value: And) -> Self {
        Self::And(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0xa6 => {
            // Read [HL]
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(And::Execute(op2).into()))
        }
        0xe6 => {
            // Fetch n8
            let op2 = cpu.fetchbyte();
            // Proceed
            Ok(Some(And::Execute(op2).into()))
        }
        0xa0..=0xa7 => {
            // Prepare op2
            let op2 = help::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op2)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(_: Byte, cpu: &mut Cpu, op2: Byte) -> Return {
    // Execute AND
    let acc = cpu.reg.a.load();
    let res = acc.bitand(op2);
    cpu.reg.a.store(res);

    // Set flags
    let flags = &mut cpu.reg.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, true);
    Flag::C.set(flags, false);
    cpu.reg.f.store(*flags);

    // Finish
    Ok(None)
}
