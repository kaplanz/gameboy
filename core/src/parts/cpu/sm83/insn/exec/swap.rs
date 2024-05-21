use remus::reg::Register;
use remus::Byte;

use super::{help, Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Swap(Swap::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Swap {
    #[default]
    Fetch,
    Execute(Byte),
    Delay,
}

impl Execute for Swap {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op1) => execute(code, cpu, op1),
            Self::Delay        => delay(code, cpu),
        }
    }
}

impl From<Swap> for Operation {
    fn from(value: Swap) -> Self {
        Self::Swap(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x36 => {
            // Read [HL]
            let op1 = cpu.readbyte();
            // Proceed
            Ok(Some(Swap::Execute(op1).into()))
        }
        0x30..=0x37 => {
            // Prepare op1
            let op1 = help::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op1)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(code: Byte, cpu: &mut Cpu, op1: Byte) -> Return {
    // Execute SWAP
    let res = ((op1 & 0xf0) >> 4) | ((op1 & 0x0f) << 4);

    // Set flags
    let flags = &mut cpu.reg.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, false);
    Flag::C.set(flags, false);
    cpu.reg.f.store(*flags);

    // Check opcode
    match code {
        0x36 => {
            // Write [HL]
            cpu.writebyte(res);
            // Proceed
            Ok(Some(Swap::Delay.into()))
        }
        0x30..=0x37 => {
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
