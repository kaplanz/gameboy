use rugby_arch::reg::Register;
use rugby_arch::Byte;

use super::{Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Ret(Ret::Check)
}

#[derive(Clone, Debug, Default)]
pub enum Ret {
    #[default]
    Check,
    Pop0,
    Pop1(Byte),
    Jump(u16),
    Done,
}

impl Execute for Ret {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Check     => check(code, cpu),
            Self::Pop0      => pop0(code, cpu),
            Self::Pop1(pc0) => pop1(code, cpu, pc0),
            Self::Jump(pc)  => jump(code, cpu, pc),
            Self::Done      => done(code, cpu),
        }
    }
}

impl From<Ret> for Operation {
    fn from(value: Ret) -> Self {
        Self::Ret(value)
    }
}

fn check(code: Byte, cpu: &mut Cpu) -> Return {
    // Evaluate condition
    let flags = &cpu.reg.f.load();
    #[rustfmt::skip]
    let cond = match code {
        0xc0 => !Flag::Z.get(flags),
        0xc8 =>  Flag::Z.get(flags),
        0xd0 => !Flag::C.get(flags),
        0xd8 =>  Flag::C.get(flags),
        0xc9 => {
            // Continue
            return pop0(code, cpu);
        }
        code => return Err(Error::Opcode(code)),
    };

    // Check condition
    if cond {
        // Proceed
        Ok(Some(Ret::Pop0.into()))
    } else {
        // Proceed
        Ok(Some(Ret::Done.into()))
    }
}

fn pop0(_: Byte, cpu: &mut Cpu) -> Return {
    // Pop lower(PC) <- [SP]
    let pc0 = cpu.popbyte();

    // Proceed
    Ok(Some(Ret::Pop1(pc0).into()))
}

fn pop1(_: Byte, cpu: &mut Cpu, pc0: Byte) -> Return {
    // Pop upper(PC) <- [SP + 1]
    let pc1 = cpu.popbyte();
    // Combine into PC
    let pc = u16::from_le_bytes([pc0, pc1]);

    // Proceed
    Ok(Some(Ret::Jump(pc).into()))
}

fn jump(_: Byte, cpu: &mut Cpu, pc: u16) -> Return {
    // Perform jump
    cpu.reg.pc.store(pc);

    // Proceed
    Ok(Some(Ret::Done.into()))
}

fn done(_: Byte, _: &mut Cpu) -> Return {
    // Finish
    Ok(None)
}
