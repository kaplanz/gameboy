use super::*;

pub const fn default() -> Operation {
    Operation::Sra(Sra::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Sra {
    #[default]
    Fetch,
    Execute(u8),
    Delay,
}

impl Execute for Sra {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Execute(op1) => execute(code, cpu, op1),
            Self::Delay => delay(code, cpu),
        }
    }
}

impl From<Sra> for Operation {
    fn from(value: Sra) -> Self {
        Self::Sra(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x2e => {
            // Read [HL]
            let op1 = cpu.readbyte();
            // Proceed
            Ok(Some(Sra::Execute(op1).into()))
        }
        0x28..=0x2f => {
            // Prepare op1
            let op1 = helpers::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op1)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(code: u8, cpu: &mut Cpu, op1: u8) -> Return {
    // Execute SRA
    let sign = op1 & 0x80;
    let carry = op1 & 0x01 != 0;
    let res = sign | (op1 >> 1);

    // Set flags
    let flags = &mut cpu.file.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, false);
    Flag::C.set(flags, carry);
    cpu.file.f.store(*flags);

    // Check opcode
    match code {
        0x2e => {
            // Write [HL]
            cpu.writebyte(res);
            // Proceed
            Ok(Some(Sra::Delay.into()))
        }
        0x28..=0x2f => {
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
