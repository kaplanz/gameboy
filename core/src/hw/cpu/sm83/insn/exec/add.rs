use enuf::Enuf;
use remus::Cell;

use super::{helpers, Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Add(Add::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Add {
    #[default]
    Fetch,
    Execute(u8),
}

impl Execute for Add {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
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

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x86 => {
            // Read [HL]
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(Add::Execute(op2).into()))
        }
        0xc6 => {
            // Fetch n8
            let op2 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Add::Execute(op2).into()))
        }
        0x80..=0x87 => {
            // Prepare op2
            let op2 = helpers::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op2)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(_: u8, cpu: &mut Cpu, op2: u8) -> Return {
    // Execute ADD
    let acc = cpu.file.a.load();
    let (res, carry) = acc.overflowing_add(op2);
    cpu.file.a.store(res);

    // Set flags
    let flags = &mut cpu.file.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, 0x0f < (acc & 0x0f) + (op2 & 0x0f));
    Flag::C.set(flags, carry);
    cpu.file.f.store(*flags);

    // Finish
    Ok(None)
}
