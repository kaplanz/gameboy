use super::*;

pub const fn default() -> Operation {
    Operation::Jp(Jp::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Jp {
    #[default]
    Fetch,
    Evaluate(u16),
    Check(u16, bool),
    Jump(u16),
}

impl Execute for Jp {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Evaluate(a16) => evaluate(code, cpu, a16),
            Self::Check(a16, cond) => check(code, cpu, a16, cond),
            Self::Jump(a16) => jump(code, cpu, a16),
        }
    }
}

impl From<Jp> for Operation {
    fn from(value: Jp) -> Self {
        Self::Jp(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0xc2 | 0xc3 | 0xca | 0xd2 | 0xda => {
            // Fetch a16
            let a16 = cpu.fetchword();
            // Proceed
            Ok(Some(Jp::Evaluate(a16).into()))
        }
        0xe9 => {
            // Load HL
            let a16 = cpu.file.hl.load(&cpu.file);
            // Continue
            jump(code, cpu, a16)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn evaluate(code: u8, cpu: &mut Cpu, a16: u16) -> Return {
    // Delay by 1 cycle

    // Evaluate condition
    let flags = &cpu.file.f.load();
    let cond = match code {
        0xc2 => !Flag::Z.get(flags),
        0xc3 => true,
        0xca => Flag::Z.get(flags),
        0xd2 => !Flag::C.get(flags),
        0xda => Flag::C.get(flags),
        code => return Err(Error::Opcode(code)),
    };

    // Proceed
    Ok(Some(Jp::Check(a16, cond).into()))
}

fn check(_: u8, _: &mut Cpu, a16: u16, cond: bool) -> Return {
    // Execute JP
    if cond {
        // Proceed
        Ok(Some(Jp::Jump(a16).into()))
    } else {
        // Finish
        Ok(None)
    }
}

fn jump(_: u8, cpu: &mut Cpu, a16: u16) -> Return {
    // Perform jump
    cpu.file.pc.store(a16);

    // Finish
    Ok(None)
}
