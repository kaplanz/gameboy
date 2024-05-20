use enuf::Enuf;
use remus::reg::Register;
use remus::Byte;

use super::{help, Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Srl(Srl::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Srl {
    #[default]
    Fetch,
    Execute(Byte),
    Delay,
}

impl Execute for Srl {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op1) => execute(code, cpu, op1),
            Self::Delay        => delay(code, cpu),
        }
    }
}

impl From<Srl> for Operation {
    fn from(value: Srl) -> Self {
        Self::Srl(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x3e => {
            // Read [HL]
            let op1 = cpu.readbyte();
            // Proceed
            Ok(Some(Srl::Execute(op1).into()))
        }
        0x38..=0x3f => {
            // Prepare op1
            let op1 = help::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op1)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(code: Byte, cpu: &mut Cpu, op1: Byte) -> Return {
    // Execute SRL
    let carry = op1 & 0x01 != 0;
    let res = 0x7f & (op1 >> 1);

    // Set flags
    let flags = &mut cpu.reg.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, false);
    Flag::C.set(flags, carry);
    cpu.reg.f.store(*flags);

    // Check opcode
    match code {
        0x3e => {
            // Write [HL]
            cpu.writebyte(res);
            // Proceed
            Ok(Some(Srl::Delay.into()))
        }
        0x38..=0x3f => {
            // Store r8
            help::set_op8(cpu, code & 0x07, res);
            // Finish
            Ok(None)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn delay(_: Byte, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
