use std::ops::BitOr;

use enuf::Enuf;
use remus::reg::Register;
use remus::Byte;

use super::{help, Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Or(Or::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Or {
    #[default]
    Fetch,
    Execute(Byte),
}

impl Execute for Or {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op2) => execute(code, cpu, op2),
        }
    }
}

impl From<Or> for Operation {
    fn from(value: Or) -> Self {
        Self::Or(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0xb6 => {
            // Read [HL]
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(Or::Execute(op2).into()))
        }
        0xf6 => {
            // Fetch [PC++]
            let op2 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Or::Execute(op2).into()))
        }
        0xb0..=0xb7 => {
            // Prepare op2
            let op2 = help::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op2)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(_: Byte, cpu: &mut Cpu, op2: Byte) -> Return {
    // Execute OR
    let acc = cpu.reg.a.load();
    let res = acc.bitor(op2);
    cpu.reg.a.store(res);

    // Set flags
    let flags = &mut cpu.reg.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, false);
    Flag::C.set(flags, false);
    cpu.reg.f.store(*flags);

    // Finish
    Ok(None)
}
