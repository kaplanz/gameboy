use remus::reg::Register;
use remus::Byte;

use super::{help, Cpu, Error, Execute, Flag, Operation, Return};

pub const fn default() -> Operation {
    Operation::Sbc(Sbc::Fetch)
}

#[derive(Clone, Debug, Default)]
pub enum Sbc {
    #[default]
    Fetch,
    Execute(Byte),
}

impl Execute for Sbc {
    #[rustfmt::skip]
    fn exec(self, code: Byte, cpu: &mut Cpu) -> Return {
        match self {
            Self::Fetch        => fetch(code, cpu),
            Self::Execute(op2) => execute(code, cpu, op2),
        }
    }
}

impl From<Sbc> for Operation {
    fn from(value: Sbc) -> Self {
        Self::Sbc(value)
    }
}

fn fetch(code: Byte, cpu: &mut Cpu) -> Return {
    // Check opcode
    match code {
        0x9e => {
            // Read [HL]
            let op2 = cpu.readbyte();
            // Proceed
            Ok(Some(Sbc::Execute(op2).into()))
        }
        0xde => {
            // Fetch n8 <- [PC++]
            let n8 = cpu.fetchbyte();
            // Proceed
            Ok(Some(Sbc::Execute(n8).into()))
        }
        0x98..=0x9f => {
            // Prepare op2
            let op2 = help::get_op8(cpu, code & 0x07);
            // Continue
            execute(code, cpu, op2)
        }
        code => Err(Error::Opcode(code)),
    }
}

fn execute(_: Byte, cpu: &mut Cpu, op2: Byte) -> Return {
    // Execute SUB
    let flags = &cpu.reg.f.load();
    let acc = cpu.reg.a.load();
    let cin = Flag::C.get(flags) as Byte;
    let (res, carry0) = acc.overflowing_sub(op2);
    let (res, carry1) = res.overflowing_sub(cin);
    cpu.reg.a.store(res);

    // Set flags
    let flags = &mut cpu.reg.f.load();
    Flag::Z.set(flags, res == 0);
    Flag::N.set(flags, true);
    Flag::H.set(flags, (op2 & 0x0f) + cin > (acc & 0x0f));
    Flag::C.set(flags, carry0 | carry1);
    cpu.reg.f.store(*flags);

    // Finish
    Ok(None)
}
