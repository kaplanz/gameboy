use super::*;
use crate::hw::pic::Interrupt;

#[derive(Clone, Debug, Default)]
pub enum Int {
    #[default]
    Execute,
    Nop,
    Push,
    Delay,
    Jump,
}

impl Execute for Int {
    fn exec(self, code: u8, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
            Self::Nop => nop(code, cpu),
            Self::Push => push(code, cpu),
            Self::Delay => delay(code, cpu),
            Self::Jump => jump(code, cpu),
        }
    }
}

impl From<Int> for Operation {
    fn from(value: Int) -> Self {
        Self::Int(value)
    }
}

fn execute(_: u8, cpu: &mut Cpu) -> Return {
    // Disable interrupts
    cpu.ime = Ime::Disabled;

    // Proceed
    Ok(Some(Int::Nop.into()))
}

fn nop(_: u8, _: &mut Cpu) -> Return {
    // Execute NOP

    // Proceed
    Ok(Some(Int::Push.into()))
}

fn push(_: u8, cpu: &mut Cpu) -> Return {
    // Push [SP] <- PC
    let pc = cpu.file.pc.load();
    cpu.pushword(pc);

    // Proceed
    Ok(Some(Int::Delay.into()))
}

fn delay(_: u8, _: &mut Cpu) -> Return {
    // Delay by 1 cycle

    // Proceed
    Ok(Some(Int::Jump.into()))
}

fn jump(code: u8, cpu: &mut Cpu) -> Return {
    // Perform jump
    let int = Interrupt::try_from(code).map_err(|_| Error::Opcode(code))?;
    cpu.file.pc.store(int.handler() as u16);

    // Finish
    Ok(None)
}
