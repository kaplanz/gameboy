use super::*;

pub const fn default() -> Operation {
    Operation::Xor(Xor::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Xor {
    #[default]
    Fetch,
    Execute(u8),
}

impl Execute for Xor {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Execute(op2) => execute(code, cpu, op2),
        }
    }
}

impl From<Xor> for Operation {
    fn from(value: Xor) -> Self {
        Self::Xor(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0xae => {
            // Read [HL]
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(Xor::Execute(op2).into()))
        }
        0xee => {
            // Fetch n8
            let op2 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Xor::Execute(op2).into()))
        }
        0xa8..=0xaf => {
            // Prepare op2
            let op2 = helpers::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op2)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(_: u8, cpu: &mut Cpu, op2: u8) -> Return {
    // Execute XOR
    let acc = cpu.file.a.load();
    let res = acc.bitxor(op2);
    cpu.file.a.store(res);

    // Set flags
    let flags = &mut cpu.file.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, false);
    Flag::H.set(flags, false);
    Flag::C.set(flags, false);
    cpu.file.f.store(*flags);

    // Finish
    Ok(None)
}
