use super::*;

pub const fn default() -> Operation {
    Operation::Rr(Rr::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Rr {
    #[default]
    Fetch,
    Execute(u8),
    Delay,
}

impl Execute for Rr {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Execute(op1) => execute(code, cpu, op1),
            Self::Delay => delay(code, cpu),
        }
    }
}

impl From<Rr> for Operation {
    fn from(value: Rr) -> Self {
        Self::Rr(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x1e => {
            // Read [HL]
            let op1 = cpu.readbyte();
            // Proceed
            Ok(Some(Rr::Execute(op1).into()))
        }
        0x18..=0x1f => {
            // Prepare op2
            let op1 = helpers::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op1)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(code: u8, cpu: &mut Cpu, op1: u8) -> Return {
    // Execute RR
    let flags = &mut cpu.file.f.load();
    let cin = Flag::C.get(flags);
    let carry = op1 & 0x01 != 0;
    let res = ((cin as u8) << 7) | op1 >> 1;

    // Set flags
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, false);
    Flag::C.set(flags, carry);
    cpu.file.f.store(*flags);

    // Check opcode
    match code {
        0x1e => {
            // Write [HL]
            cpu.writebyte(res);
            // Proceed
            Ok(Some(Rr::Delay.into()))
        }
        0x18..=0x1f => {
            // Store r8
            helpers::set_op8(cpu, code & 0x07, res);
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
