use super::*;

pub const fn default() -> Operation {
    Operation::Ret(Ret::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Ret {
    #[default]
    Fetch,
    Check(bool),
    Pop,
    Delay(u16),
    Jump(u16),
}

impl Execute for Ret {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch => fetch(code, cpu),
            Self::Check(cond) => check(code, cpu, cond),
            Self::Pop => pop(code, cpu),
            Self::Delay(pc) => delay(code, cpu, pc),
            Self::Jump(pc) => jump(code, cpu, pc),
        }
    }
}

impl From<Ret> for Operation {
    fn from(value: Ret) -> Self {
        Self::Ret(value)
    }
}

fn fetch(code: u8, cpu: &mut Cpu) -> Return {
    // Evaluate condition
    let flags = &cpu.file.f.load();
    let cond = match code {
        0xc0 => !Flag::Z.get(flags),
        0xc8 => Flag::Z.get(flags),
        0xc9 => true,
        0xd0 => !Flag::C.get(flags),
        0xd8 => Flag::C.get(flags),
        code => return Err(Error::Opcode(code)),
    };

    if code == 0xc9 {
        // Continue
        check(code, cpu, cond)
    } else {
        // Proceed
        Ok(Some(Ret::Check(cond).into()))
    }
}

fn check(_: u8, _: &mut Cpu, cond: bool) -> Return {
    // Execute RET
    if cond {
        // Proceed
        Ok(Some(Ret::Pop.into()))
    } else {
        // Finish
        Ok(None)
    }
}

fn pop(_: u8, cpu: &mut Cpu) -> Return {
    // Pop PC <- [SP]
    let pc = cpu.popword();

    // Proceed
    Ok(Some(Ret::Delay(pc).into()))
}

fn delay(_: u8, _: &mut Cpu, pc: u16) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Ret::Jump(pc).into()))
}

fn jump(_: u8, cpu: &mut Cpu, pc: u16) -> Return {
    // Perform jump
    cpu.file.pc.store(pc);

    // Finish
    Ok(None)
}
