use rugby_arch::Byte;

use super::{help, Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Res(Res::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Res {
    #[default]
    Fetch,
    Execute(Byte),
    Delay,
}

impl Execute for Res {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op2) => execute(code, cpu, op2),
            Self::Delay        => delay(code, cpu),
        }
    }
}

impl From<Res> for Operation {
    fn from(value: Res) -> Self {
        Self::Res(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x86 | 0x8e | 0x96 | 0x9e | 0xa6 | 0xae | 0xb6 | 0xbe => {
            // Read [HL]
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(Res::Execute(op2).into()))
        }
        0x80..=0xbf => {
            // Prepare op2
            let op2 = help::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op2)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(code: Byte, cpu: &mut Cpu, op2: Byte) -> Return {
    // Execute RES
    let op1 = (code & 0x38) >> 3;
    let mask = !(0b1 << op1);
    let res = mask & op2;

    // Check opcode
    match code {
        0x86 | 0x8e | 0x96 | 0x9e | 0xa6 | 0xae | 0xb6 | 0xbe => {
            // Write [HL]
            cpu.writebyte(res);
            // Proceed
            Ok(Some(Res::Delay.into()))
        }
        0x80..=0xbf => {
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
