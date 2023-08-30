use super::*;

pub const fn default() -> Operation {
    Operation::Call(Call::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Call {
    #[default]
    Fetch,
    Evaluate(u16),
    Check(u16, bool),
    Push(u16),
    Delay(u16),
    Jump(u16),
}

impl Execute for Call {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Evaluate(a16) => evaluate(code, cpu, a16),
            Self::Check(a16, cond) => check(code, cpu, a16, cond),
            Self::Push(a16) => push(code, cpu, a16),
            Self::Delay(a16) => delay(code, cpu, a16),
            Self::Jump(a16) => jump(code, cpu, a16),
        }
    }
}

impl From<Call> for Operation {
    fn from(value: Call) -> Self {
        Self::Call(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0xc4 | 0xcc | 0xcd | 0xd4 | 0xdc => {
            // Fetch u16
            let a16 = cpu.fetchword();
            // Proceed
            Ok(Some(Call::Evaluate(a16).into()))
        }
        code => Err(Error::Opcode(code)),
    }
}

fn evaluate(code: u8, cpu: &mut Cpu, a16: u16) -> Return {
    // Delay by 1 cycle

    // Evaluate condition
    let flags = &cpu.file.f.load();
    let cond = match code {
        0xc4 => !Flag::Z.get(flags),
        0xcc => Flag::Z.get(flags),
        0xcd => true,
        0xd4 => !Flag::C.get(flags),
        0xdc => Flag::C.get(flags),
        code => return Err(Error::Opcode(code)),
    };

    // Proceed
    Ok(Some(Call::Check(a16, cond).into()))
}

fn check(_: u8, _: &mut Cpu, a16: u16, cond: bool) -> Return {
    // Execute CALL
    if cond {
        // Proceed
        Ok(Some(Call::Push(a16).into()))
    } else {
        // Finish
        Ok(None)
    }
}

fn push(_: u8, cpu: &mut Cpu, a16: u16) -> Return {
    // Push [SP] <- PC
    let pc = cpu.file.pc.load();
    cpu.pushword(pc);

    // Proceed
    Ok(Some(Call::Delay(a16).into()))
}

fn delay(_: u8, _: &mut Cpu, a16: u16) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Call::Jump(a16).into()))
}

fn jump(_: u8, cpu: &mut Cpu, a16: u16) -> Return {
    // Perform jump
    cpu.file.pc.store(a16);

    // Finish
    Ok(None)
}
