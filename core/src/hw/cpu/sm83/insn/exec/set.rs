use super::{help, Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Set(Set::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Set {
    #[default]
    Fetch,
    Execute(u8),
    Delay,
}

impl Execute for Set {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op2) => execute(code, cpu, op2),
            Self::Delay        => delay(code, cpu),
        }
    }
}

impl From<Set> for Operation {
    fn from(value: Set) -> Self {
        Self::Set(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0xc6 | 0xce | 0xd6 | 0xde | 0xe6 | 0xee | 0xf6 | 0xfe => {
            // Read [HL]
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(Set::Execute(op2).into()))
        }
        0xc0..=0xff => {
            // Prepare op2
            let op2 = help::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op2)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(code: u8, cpu: &mut Cpu, op2: u8) -> Return {
    // Execute SET
    let op1 = (code & 0x38) >> 3;
    let mask = !(0b1 << op1);
    let res = (mask & op2) | !mask;

    // Check opcode
    match code {
        0xc6 | 0xce | 0xd6 | 0xde | 0xe6 | 0xee | 0xf6 | 0xfe => {
            // Write [HL]
            cpu.writebyte(res);
            // Proceed
            Ok(Some(Set::Delay.into()))
        }
        0xc0..=0xff => {
            // Store r8
            help::set_op8(cpu, code & 0x07, res);
            // Finish
            Ok(None)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn delay(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Finish
    Ok(None)
}
