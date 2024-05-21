use remus::reg::Register;
use remus::Byte;

use super::{help, Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Cp(Cp::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Cp {
    #[default]
    Fetch,
    Execute(Byte),
}

impl Execute for Cp {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op2) => execute(code, cpu, op2),
        }
    }
}

impl From<Cp> for Operation {
    fn from(value: Cp) -> Self {
        Self::Cp(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0xbe => {
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(Cp::Execute(op2).into()))
        }
        0xfe => {
            let op2 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Cp::Execute(op2).into()))
        }
        0xb8..=0xbf => {
            let op2 = help::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op2)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(_: Byte, cpu: &mut Cpu, op2: Byte) -> Return {
    // Execute CP
    let acc = cpu.reg.a.load();
    let (res, carry) = acc.overflowing_sub(op2);

    // Set flags
    let flags = &mut cpu.reg.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, true);
    Flag::H.set(flags, (op2 & 0x0f) > (acc & 0x0f));
    Flag::C.set(flags, carry);
    cpu.reg.f.store(*flags);

    // Finish
    Ok(None)
}
