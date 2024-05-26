use rugby_arch::Byte;

use super::{Cpu, Error, Execute, Operation, Return};

pub const fn default() -> Operation {
    Operation::Stop(Stop::Execute)
}

#[derive(Clone, Debug, Default)]
pub enum Stop {
    #[default]
    Execute,
}

impl Execute for Stop {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Execute => execute(code, cpu),
        }
    }
}

impl From<Stop> for Operation {
    fn from(value: Stop) -> Self {
        Self::Stop(value)
    }
}

fn execute(code: Byte, _: &mut Cpu) -> Return {
    // Check opcode
    if code != 0x10 {
        return Err(Error::Opcode(code));
    }

    // Execute STOP
    // <https://gbdev.io/pandocs/imgs/gb_stop.png>
    #[cfg(debug_assertions)]
    return Err(Error::Unimplemented(code));

    // Finish
    #[allow(unreachable_code)]
    Ok(None)
}
