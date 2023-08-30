use super::*;

pub const fn default() -> Operation {
    Operation::Decw(Decw::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Decw {
    #[default]
    Fetch,
    Execute(u16),
}

impl Execute for Decw {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Execute(op1) => execute(code, cpu, op1),
        }
    }
}

impl From<Decw> for Operation {
    fn from(value: Decw) -> Self {
        Self::Decw(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    let op1 = match code {
        0x0b => cpu.file.bc.load(&cpu.file),
        0x1b => cpu.file.de.load(&cpu.file),
        0x2b => cpu.file.hl.load(&cpu.file),
        0x3b => cpu.file.sp.load(),
        code => return Err(Error::Opcode(code)),
    };

    // Proceed
    Ok(Some(Decw::Execute(op1).into()))
}

fn execute(code: u8, cpu: &mut Cpu, op1: u16) -> Return {
    // Execute DECW
    let res = op1.wrapping_sub(1);
    match code {
        0x0b => {
            let bc = cpu.file.bc;
            bc.store(&mut cpu.file, res);
        }
        0x1b => {
            let de = cpu.file.de;
            de.store(&mut cpu.file, res);
        }
        0x2b => {
            let hl = cpu.file.hl;
            hl.store(&mut cpu.file, res);
        }
        0x3b => cpu.file.sp.store(res),
        code => return Err(Error::Opcode(code)),
    }

    // Finish
    Ok(None)
}
