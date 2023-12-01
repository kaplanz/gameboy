use enuf::Enuf;
use remus::Cell;

use super::{helpers, Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Bit(Bit::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Bit {
    #[default]
    Fetch,
    Execute(u8),
}

impl Execute for Bit {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op2) => execute(code, cpu, op2),
        }
    }
}

impl From<Bit> for Operation {
    fn from(value: Bit) -> Self {
        Self::Bit(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x46 | 0x4e | 0x56 | 0x5e | 0x66 | 0x6e | 0x76 | 0x7e => {
            // Read [HL]
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(Bit::Execute(op2).into()))
        }
        0x40..=0x7f => {
            // Prepare op2
            let op2 = helpers::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op2)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(code: u8, cpu: &mut Cpu, op2: u8) -> Return {
    // Execute BIT
    let op1 = (code & 0x38) >> 3;
    let res = (0b1 << op1) & op2;

    // Set flags
    let flags = &mut cpu.file.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, true);
    cpu.file.f.store(*flags);

    // Finish
    Ok(None)
}
