use enuf::Enuf;
use remus::reg::Register;
use remus::Byte;

use super::{help, Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Add(Add::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Add {
    #[default]
    Fetch,
    Execute(Byte),
}

impl Execute for Add {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op2) => execute(code, cpu, op2),
        }
    }
}

impl From<Add> for Operation {
    fn from(value: Add) -> Self {
        Self::Add(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x86 => {
            // Read [HL]
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(Add::Execute(op2).into()))
        }
        0xc6 => {
            // Fetch n8 <- [PC++]
            let op2 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Add::Execute(op2).into()))
        }
        0x80..=0x87 => {
            // Prepare op2
            let op2 = help::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op2)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(_: Byte, cpu: &mut Cpu, op2: Byte) -> Return {
    // Execute ADD
    let acc = cpu.reg.a.load();
    let (res, carry) = acc.overflowing_add(op2);
    cpu.reg.a.store(res);

    // Set flags
    let flags = &mut cpu.reg.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, 0x0f < (acc & 0x0f) + (op2 & 0x0f));
    Flag::C.set(flags, carry);
    cpu.reg.f.store(*flags);

    // Finish
    Ok(None)
}
