//! Instruction execution implementations.

#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::wildcard_imports)]

use std::ops::{BitAnd, BitOr, BitXor};

use enuf::Enuf;
use remus::Address;

use super::{helpers, Cpu, Flag, Ime, Instruction, Status};

/// Arithmetic add with carry.
pub mod adc {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x8e => {
                // Read (HL)
                let op2 = cpu.readbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xce => {
                // Fetch d8
                let op2 = cpu.fetchbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0x88..=0x8f => {
                // Prepare op2
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op2);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute ADC
        let acc = *cpu.file.a;
        let op2 = insn.stack.pop().unwrap();
        let cin = Flag::C.get(&cpu.file.f) as u8;
        let (res, carry0) = acc.overflowing_add(op2);
        let (res, carry1) = res.overflowing_add(cin);
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, 0x0f < (acc & 0x0f) + (op2 & 0x0f) + cin);
        Flag::C.set(flags, carry0 | carry1);

        // Finish
        None
    }
}

/// Arithmetic add.
pub mod add {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x86 => {
                // Read (HL)
                let op2 = cpu.readbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xc6 => {
                // Fetch d8
                let op2 = cpu.fetchbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0x80..=0x87 => {
                // Prepare op2
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op2);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute ADD
        let acc = *cpu.file.a;
        let op2 = insn.stack.pop().unwrap();
        let (res, carry) = acc.overflowing_add(op2);
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, 0x0f < (acc & 0x0f) + (op2 & 0x0f));
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

/// Arithmetic add wide (16-bit).
pub mod addw {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x09 | 0x19 | 0x29 | 0x39 => {
                let op1 = cpu.file.hl.load(&cpu.file);
                let op2 = match insn.opcode {
                    0x09 => cpu.file.bc.load(&cpu.file),
                    0x19 => cpu.file.de.load(&cpu.file),
                    0x29 => cpu.file.hl.load(&cpu.file),
                    0x39 => *cpu.file.sp,
                    _ => panic!("Illegal instruction."),
                };
                insn.stack.extend(op1.to_le_bytes());
                insn.stack.extend(op2.to_le_bytes());
                insn.exec = done;
                Some(insn)
            }
            0xe8 => {
                // Fetch r8
                let r8 = cpu.fetchbyte();
                insn.stack.push(r8);
                // Proceed
                insn.exec = exec_0xe8_1;
                Some(insn)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute ADDW
        let op1 = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        let op2 = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        let (res, carry) = op1.overflowing_add(op2);
        let hl = cpu.file.hl;
        hl.store(&mut cpu.file, res);

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::N.set(flags, false);
        Flag::H.set(flags, 0x0fff < (op1 & 0x0fff) + (op2 & 0x0fff));
        Flag::C.set(flags, carry);

        // Finish
        None
    }

    pub fn exec_0xe8_1(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        insn.exec = exec_0xe8_2;
        Some(insn)
    }

    pub fn exec_0xe8_2(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        insn.exec = done_0xe8;
        Some(insn)
    }

    pub fn done_0xe8(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute ADDW
        let op1 = *cpu.file.sp;
        let op2 = insn.stack.pop().unwrap() as i8 as u16;
        let res = op1.wrapping_add(op2);
        *cpu.file.sp = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, false);
        Flag::N.set(flags, false);
        Flag::H.set(flags, 0x000f < (op1 & 0x000f) + (op2 & 0x000f));
        Flag::C.set(flags, 0x00ff < (op1 & 0x00ff) + (op2 & 0x00ff));

        // Finish
        None
    }
}

/// Logical AND.
pub mod and {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0xa6 => {
                // Read (HL)
                let op2 = cpu.readbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xe6 => {
                // Fetch d8
                let op2 = cpu.fetchbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xa0..=0xa7 => {
                // Prepare op2
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op2);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute AND
        let acc = *cpu.file.a;
        let op2 = insn.stack.pop().unwrap();
        let res = acc.bitand(op2);
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, true);
        Flag::C.set(flags, false);

        // Finish
        None
    }
}

/// Test bit.
pub mod bit {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x46 | 0x4e | 0x56 | 0x5e | 0x66 | 0x6e | 0x76 | 0x7e => {
                // Read (HL)
                let op2 = cpu.readbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0x40..=0x7f => {
                // Prepare op2
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op2);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute BIT
        let op1 = (insn.opcode & 0x38) >> 3;
        let op2 = insn.stack.pop().unwrap();
        let res = (0b1 << op1) & op2;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, true);

        // Finish
        None
    }
}

/// Call subroutine.
pub mod call {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0xc4 | 0xcc | 0xcd | 0xd4 | 0xdc => {
                // Fetch u16
                let op1 = cpu.fetchword();
                insn.stack.extend(op1.to_le_bytes());
                // Proceed
                insn.exec = evaluate;
                Some(insn)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn evaluate(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to fetch a u16.

        // Evaluate condition
        let flags = &mut cpu.file.f;
        let cond = match insn.opcode {
            0xc4 => !Flag::Z.get(flags),
            0xcc => Flag::Z.get(flags),
            0xcd => true,
            0xd4 => !Flag::C.get(flags),
            0xdc => Flag::C.get(flags),
            _ => panic!("Illegal instruction."),
        };
        insn.stack.push(u8::from(cond));

        // Proceed
        insn.exec = check;
        Some(insn)
    }

    pub fn check(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Execute CALL
        let cond = insn.stack.pop().unwrap() != 0;
        if cond {
            // Proceed
            insn.exec = push;
            Some(insn)
        } else {
            // Finish
            None
        }
    }

    pub fn push(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Push SP
        cpu.pushword(*cpu.file.pc);

        // Proceed
        insn.exec = delay;
        Some(insn)
    }

    pub fn delay(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to push a u16.

        // Proceed
        insn.exec = jump;
        Some(insn)
    }

    pub fn jump(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let op1 = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        *cpu.file.pc = op1;

        // Finish
        None
    }
}

/// Complement carry flag.
pub mod ccf {
    use super::*;

    pub fn start(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0x3f, "Illegal instruction.");

        // Execute CCF
        let flags = &mut cpu.file.f;
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        let carry = Flag::C.get(flags);
        Flag::C.set(flags, !carry);

        // Finish
        None
    }
}

/// Compare.
pub mod cp {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0xbe => {
                let op2 = cpu.readbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xfe => {
                let op2 = cpu.fetchbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xb8..=0xbf => {
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op2);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute CP
        let acc = *cpu.file.a;
        let op2 = insn.stack.pop().unwrap();
        let (res, carry) = acc.overflowing_sub(op2);

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, true);
        Flag::H.set(flags, (op2 & 0x0f) > (acc & 0x0f));
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

/// Complement.
pub mod cpl {
    use super::*;

    pub fn start(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0x2f, "Illegal instruction.");

        // Execute CPL
        let acc = *cpu.file.a;
        let res = !acc;
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::N.set(flags, true);
        Flag::H.set(flags, true);

        // Finish
        None
    }
}

/// Decimal adjust after addition.
pub mod daa {
    use super::*;

    pub fn start(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0x27, "Illegal instruction.");

        // Execute DAA
        let didsub = Flag::N.get(&cpu.file.f);
        let hcarry = Flag::H.get(&cpu.file.f);
        let mut carry = Flag::C.get(&cpu.file.f);
        let mut adj = 0i8;
        let acc = *cpu.file.a;
        if hcarry || (!didsub && (acc & 0x0f) > 0x09) {
            adj |= 0x06;
        }
        if carry || (!didsub && acc > 0x99) {
            adj |= 0x60;
            carry = true;
        }
        adj = if didsub { -adj } else { adj };
        let res = (acc as i8).wrapping_add(adj) as u8;
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

/// Decrement.
pub mod dec {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x35 => {
                let op1 = cpu.readbyte();
                insn.stack.push(op1);
                insn.exec = done;
                Some(insn)
            }
            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x3d => {
                let op1 = match insn.opcode {
                    0x05 => *cpu.file.b,
                    0x0d => *cpu.file.c,
                    0x15 => *cpu.file.d,
                    0x1d => *cpu.file.e,
                    0x25 => *cpu.file.h,
                    0x2d => *cpu.file.l,
                    0x3d => *cpu.file.a,
                    _ => panic!("Illegal instruction."),
                };
                insn.stack.push(op1);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    #[allow(clippy::verbose_bit_mask)]
    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute DEC
        let op1 = insn.stack.pop().unwrap();
        let res = op1.wrapping_sub(1);

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, true);
        Flag::H.set(flags, op1 & 0x0f == 0);

        // Check opcode
        match insn.opcode {
            0x35 => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x3d => {
                // Write X
                *match insn.opcode {
                    0x05 => &mut *cpu.file.b,
                    0x0d => &mut *cpu.file.c,
                    0x15 => &mut *cpu.file.d,
                    0x1d => &mut *cpu.file.e,
                    0x25 => &mut *cpu.file.h,
                    0x2d => &mut *cpu.file.l,
                    0x3d => &mut *cpu.file.a,
                    _ => panic!("Illegal instruction."),
                } = res;
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

/// Decrement wide (16-bit).
pub mod decw {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        let op1 = match insn.opcode {
            0x0b => cpu.file.bc.load(&cpu.file),
            0x1b => cpu.file.de.load(&cpu.file),
            0x2b => cpu.file.hl.load(&cpu.file),
            0x3b => *cpu.file.sp,
            _ => panic!("Illegal instruction."),
        };
        insn.stack.extend(op1.to_le_bytes());
        insn.exec = done;

        Some(insn)
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute DECW
        let op1 = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        let res = op1.wrapping_sub(1);
        match insn.opcode {
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
            0x3b => *cpu.file.sp = res,
            _ => panic!("Illegal instruction."),
        }

        // Finish
        None
    }
}

/// Disable interrupts.
pub mod di {
    use super::*;

    pub fn start(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0xf3, "Illegal instruction.");

        // Execute DI
        cpu.ime = Ime::Disabled;

        // Finish
        None
    }
}

/// Enable interrupts.
pub mod ei {
    use super::*;

    pub fn start(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0xfb, "Illegal instruction.");

        // Execute EI
        cpu.ime = Ime::WillEnable;

        // Finish
        None
    }
}

/// Halt CPU.
pub mod halt {
    use super::*;

    pub fn start(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0x76, "Illegal instruction.");

        // Perform HALT bug
        if !cpu.ime.enabled() && cpu.pic.borrow().int().is_some() {
            cpu.halt_bug = true;
        } else {
            // Execute HALT
            cpu.status = Status::Halted;
        }

        // Finish
        None
    }
}

/// Increment.
pub mod inc {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x34 => {
                let op1 = cpu.readbyte();
                insn.stack.push(op1);
                insn.exec = done;
                Some(insn)
            }
            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x3c => {
                let op1 = match insn.opcode {
                    0x04 => *cpu.file.b,
                    0x0c => *cpu.file.c,
                    0x14 => *cpu.file.d,
                    0x1c => *cpu.file.e,
                    0x24 => *cpu.file.h,
                    0x2c => *cpu.file.l,
                    0x3c => *cpu.file.a,
                    _ => panic!("Illegal instruction."),
                };
                insn.stack.push(op1);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    #[allow(clippy::verbose_bit_mask)]
    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute INC
        let op1 = insn.stack.pop().unwrap();
        let res = op1.wrapping_add(1);

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, res & 0x0f == 0);

        // Check opcode
        match insn.opcode {
            0x34 => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x3c => {
                // Write X
                *match insn.opcode {
                    0x04 => &mut *cpu.file.b,
                    0x0c => &mut *cpu.file.c,
                    0x14 => &mut *cpu.file.d,
                    0x1c => &mut *cpu.file.e,
                    0x24 => &mut *cpu.file.h,
                    0x2c => &mut *cpu.file.l,
                    0x3c => &mut *cpu.file.a,
                    _ => panic!("Illegal instruction."),
                } = res;
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

/// Increment wide (16-bit).
pub mod incw {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        let op1 = match insn.opcode {
            0x03 => cpu.file.bc.load(&cpu.file),
            0x13 => cpu.file.de.load(&cpu.file),
            0x23 => cpu.file.hl.load(&cpu.file),
            0x33 => *cpu.file.sp,
            _ => panic!("Illegal instruction."),
        };
        insn.stack.extend(op1.to_le_bytes());
        insn.exec = done;

        Some(insn)
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute INCW
        let op1 = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        let res = op1.wrapping_add(1);
        match insn.opcode {
            0x03 => {
                let bc = cpu.file.bc;
                bc.store(&mut cpu.file, res);
            }
            0x13 => {
                let de = cpu.file.de;
                de.store(&mut cpu.file, res);
            }
            0x23 => {
                let hl = cpu.file.hl;
                hl.store(&mut cpu.file, res);
            }
            0x33 => *cpu.file.sp = res,
            _ => panic!("Illegal instruction."),
        }

        // Finish
        None
    }
}

/// Interrupt service routine.
pub mod int {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Disable interrupts
        cpu.ime = Ime::Disabled;

        // Proceed
        insn.exec = nop;
        Some(insn)
    }

    pub fn nop(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Execute NOP

        // Proceed
        insn.exec = push;
        Some(insn)
    }

    pub fn push(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Push SP
        cpu.pushword(*cpu.file.pc);

        // Proceed
        insn.exec = delay;
        Some(insn)
    }

    pub fn delay(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to push a u16.

        // Proceed
        insn.exec = jump;
        Some(insn)
    }

    pub fn jump(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let addr = insn.stack.pop().unwrap() as u16;
        *cpu.file.pc = addr;

        // Finish
        None
    }
}

/// Jump.
pub mod jp {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0xc2 | 0xc3 | 0xca | 0xd2 | 0xda => {
                // Fetch u16
                let op1 = cpu.fetchword();
                insn.stack.extend(op1.to_le_bytes());
                // Proceed
                insn.exec = evaluate;
                Some(insn)
            }
            0xe9 => {
                // Read HL
                let op1 = cpu.file.hl.load(&cpu.file);
                insn.stack.extend(op1.to_le_bytes());
                // Continue
                jump(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn evaluate(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to fetch a u16.

        // Evaluate condition
        let flags = &mut cpu.file.f;
        let cond = match insn.opcode {
            0xc2 => !Flag::Z.get(flags),
            0xc3 => true,
            0xca => Flag::Z.get(flags),
            0xd2 => !Flag::C.get(flags),
            0xda => Flag::C.get(flags),
            _ => panic!("Illegal instruction."),
        };
        insn.stack.push(cond as u8);

        // Proceed
        insn.exec = check;
        Some(insn)
    }

    pub fn check(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Execute JP
        let cond = insn.stack.pop().unwrap() != 0;
        if cond {
            // Proceed
            insn.exec = jump;
            Some(insn)
        } else {
            // Finish
            None
        }
    }

    pub fn jump(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let op1 = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        *cpu.file.pc = op1;

        // Finish
        None
    }
}

/// Jump relative.
pub mod jr {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
                // Fetch u8
                let op1 = cpu.fetchbyte();
                insn.stack.push(op1);
                // Continue
                evaluate(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn evaluate(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Evaluate condition
        let flags = &mut cpu.file.f;
        let cond = match insn.opcode {
            0x18 => true,
            0x20 => !Flag::Z.get(flags),
            0x28 => Flag::Z.get(flags),
            0x30 => !Flag::C.get(flags),
            0x38 => Flag::C.get(flags),
            _ => panic!("Illegal instruction."),
        };
        insn.stack.push(cond as u8);

        // Proceed
        insn.exec = check;
        Some(insn)
    }

    pub fn check(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Execute JR
        let cond = insn.stack.pop().unwrap() != 0;
        if cond {
            // Proceed
            insn.exec = jump;
            Some(insn)
        } else {
            // Finish
            None
        }
    }

    pub fn jump(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let pc = *cpu.file.pc as i16;
        let op1 = insn.stack.pop().unwrap() as i8 as i16;
        let res = pc.wrapping_add(op1) as u16;
        *cpu.file.pc = res;

        // Finish
        None
    }
}

/// Load.
pub mod ld {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x02 | 0x12 | 0x22 | 0x32 => {
                // Execute LD (XY), A
                let addr = match insn.opcode {
                    0x02 => cpu.file.bc,
                    0x12 => cpu.file.de,
                    0x22 | 0x32 => cpu.file.hl,
                    _ => panic!("Illegal instruction."),
                }
                .load(&cpu.file);
                insn.stack.extend(addr.to_le_bytes());
                let op2 = *cpu.file.a;
                cpu.write(addr, op2);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x0a | 0x1a | 0x2a | 0x3a => {
                // Load (XY)
                let addr = match insn.opcode {
                    0x0a => cpu.file.bc,
                    0x1a => cpu.file.de,
                    0x2a | 0x3a => cpu.file.hl,
                    _ => panic!("Illegal instruction."),
                }
                .load(&cpu.file);
                insn.stack.extend(addr.to_le_bytes());
                let op2 = cpu.read(addr);
                insn.stack.push(op2);
                // Proceed
                insn.exec = execute;
                Some(insn)
            }
            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x3e => {
                // Fetch d8
                let op2 = cpu.fetchbyte();
                insn.stack.push(op2);
                // Proceed
                insn.exec = execute;
                Some(insn)
            }
            0x36 => {
                // Fetch d8
                let op2 = cpu.fetchbyte();
                insn.stack.push(op2);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x46 | 0x4e | 0x56 | 0x5e | 0x66 | 0x6e | 0x7e => {
                // Load (HL)
                let op2 = cpu.readbyte();
                insn.stack.push(op2);
                // Proceed
                insn.exec = execute;
                Some(insn)
            }
            0x76 => panic!("Illegal instruction."),
            0x70..=0x77 => {
                // Execute LD (HL), X
                let addr = cpu.file.hl.load(&cpu.file);
                insn.stack.extend(addr.to_le_bytes());
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                cpu.write(addr, op2);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x40..=0x7f => {
                // Execute LD X, Y
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op2);
                // Continue
                execute(insn, cpu)
            }
            0xea | 0xfa => {
                // Fetch a16
                let addr = cpu.fetchword();
                insn.stack.extend(addr.to_le_bytes());
                // Proceed
                insn.exec = delay_a16;
                Some(insn)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: Since the memory bus is busy with the write, we must fetch the
        //       next instruction this cycle instead.

        // Perform inc/dec on HL
        #[allow(clippy::match_same_arms)]
        match insn.opcode {
            0x02 | 0x12 => {
                // Continue
                done(insn, cpu)
            }
            0x22 | 0x32 => {
                // Continue
                inc_dec_hl(insn, cpu)
            }
            0x36 => {
                // Execute (HL), d8
                let op2 = insn.stack.pop().unwrap();
                cpu.writebyte(op2);
                // Proceed
                insn.exec = done;
                Some(insn)
            }
            0x70 | 0x71 | 0x72 | 0x73 | 0x74 | 0x75 | 0x77 => {
                // Continue
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn execute(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute LD X, {Y, d8, (HL)}
        let op1 = match insn.opcode {
            0x06 | 0x40..=0x47 => &mut *cpu.file.b,
            0x0e | 0x48..=0x4f => &mut *cpu.file.c,
            0x16 | 0x50..=0x57 => &mut *cpu.file.d,
            0x1e | 0x58..=0x5f => &mut *cpu.file.e,
            0x26 | 0x60..=0x67 => &mut *cpu.file.h,
            0x2e | 0x68..=0x6f => &mut *cpu.file.l,
            0x0a | 0x1a | 0x2a | 0x3a | 0x3e | 0x78..=0x7f | 0xf2 => &mut *cpu.file.a,
            _ => panic!("Illegal instruction."),
        };
        let op2 = insn.stack.pop().unwrap();
        *op1 = op2;

        // Continue
        match insn.opcode {
            0x2a | 0x3a => inc_dec_hl(insn, cpu),
            _ => done(insn, cpu),
        }
    }

    pub fn done(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Finish
        None
    }

    pub fn inc_dec_hl(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform inc/dec on HL
        let addr = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        let hl = cpu.file.hl;
        match insn.opcode {
            0x22 | 0x2a => hl.store(&mut cpu.file, addr.wrapping_add(1)),
            0x32 | 0x3a => hl.store(&mut cpu.file, addr.wrapping_sub(1)),
            _ => panic!("Illegal instruction."),
        }

        // Continue
        done(insn, cpu)
    }

    pub fn delay_a16(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to read an a16.

        // Proceed
        insn.exec = delay_rw;
        Some(insn)
    }

    pub fn delay_rw(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        let addr = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        match insn.opcode {
            0xea => {
                // Execute LD (a16), A
                let op2 = *cpu.file.a;
                cpu.write(addr, op2);
            }
            0xfa => {
                // Execute LD A, (a16)
                let op2 = cpu.read(addr);
                *cpu.file.a = op2;
            }
            _ => panic!("Illegal instruction."),
        }

        // Proceed
        insn.exec = done;
        Some(insn)
    }
}

/// Load wide (16-bit).
pub mod ldw {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x01 | 0x11 | 0x21 | 0x31 => {
                // Fetch d16
                let op2 = cpu.fetchword();
                insn.stack.extend(op2.to_le_bytes());
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x08 => {
                // Fetch a16
                let addr = cpu.fetchword();
                insn.stack.extend(addr.to_le_bytes());
                // Proceed
                insn.exec = delay_0x08_1;
                Some(insn)
            }
            0xf8 => {
                // Fetch r8
                let r8 = cpu.fetchbyte();
                insn.stack.push(r8);
                // Proceed
                insn.exec = delay_0xf8;
                Some(insn)
            }
            0xf9 => {
                // Read HL
                let op2 = (cpu.file.hl.load)(&cpu.file);
                insn.stack.extend(op2.to_le_bytes());
                // Proceed
                insn.exec = done;
                Some(insn)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to read a d16.

        // Proceed
        insn.exec = done;
        Some(insn)
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute LDW
        let op2 = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        match insn.opcode {
            0x01 => (cpu.file.bc.store)(&mut cpu.file, op2),
            0x11 => (cpu.file.de.store)(&mut cpu.file, op2),
            0x21 => (cpu.file.hl.store)(&mut cpu.file, op2),
            0x31 | 0xf9 => *cpu.file.sp = op2,
            _ => panic!("Illegal instruction."),
        }

        // Finish
        None
    }

    pub fn delay_0x08_1(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to read an a16.

        // Proceed
        insn.exec = delay_0x08_2;
        Some(insn)
    }

    pub fn delay_0x08_2(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Write the SP into the a16
        let mut addr = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        // Read the value of SP
        let sp = &cpu.file.sp;
        let sp0 = sp.read(0);
        let sp1 = sp.read(1);
        // Write it at a16
        cpu.write(addr, sp0);
        addr = addr.wrapping_add(1);
        cpu.write(addr, sp1);

        // Proceed
        insn.exec = delay_0x08_3;
        Some(insn)
    }

    pub fn delay_0x08_3(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to read an a16.

        // Proceed
        insn.exec = done_0x08;
        Some(insn)
    }

    pub fn done_0x08(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the delay needed to fetch the next instruction.

        // Finish
        None
    }

    pub fn delay_0xf8(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute LD HL, SP + r8
        let sp = *cpu.file.sp;
        let r8 = insn.stack.pop().unwrap() as i8 as u16;
        let res = sp.wrapping_add(r8);
        let hl = cpu.file.hl;
        hl.store(&mut cpu.file, res);

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, false);
        Flag::N.set(flags, false);
        Flag::H.set(flags, 0x000f < (sp & 0x000f) + (r8 & 0x000f));
        Flag::C.set(flags, 0x00ff < (sp & 0x00ff) + (r8 & 0x00ff));

        // Proceed
        insn.exec = done_0xf8;
        Some(insn)
    }

    pub fn done_0xf8(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the delay needed to fetch the next instruction.

        // Finish
        None
    }
}

/// Load high.
pub mod ldh {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0xe0 | 0xf0 => {
                // Fetch a8
                let a8 = cpu.fetchbyte();
                insn.stack.push(a8);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0xe2 | 0xf2 => {
                // Read C
                let a8 = *cpu.file.c;
                insn.stack.push(a8);
                // Proceed
                delay(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Calculate absolute address from relative
        let addr = 0xff00 | insn.stack.pop().unwrap() as u16;

        // Perform a read/write to the address
        match insn.opcode {
            0xe0 | 0xe2 => {
                // Execute LD(H?) (a8|C), A
                let op2 = *cpu.file.a;
                cpu.write(addr, op2);
            }
            0xf0 | 0xf2 => {
                // Execute LD(H?) A, (a8|C)
                *cpu.file.a = cpu.read(addr);
            }
            _ => panic!("Illegal instruction."),
        }

        // Proceed
        insn.exec = done;
        Some(insn)
    }

    pub fn done(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This either represents the cycle needed to perform the load, or
        //       a delay cycle due to a previous write (as we fetch the next
        //       instruction).

        // Finish
        None
    }
}

/// No operation.
pub mod nop {
    use super::*;

    pub fn start(insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0x00, "Illegal instruction.");

        // Finish
        None
    }
}

/// Logical OR.
pub mod or {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0xb6 => {
                let op2 = cpu.readbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xf6 => {
                let op2 = cpu.fetchbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xb0..=0xb7 => {
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op2);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute OR
        let acc = *cpu.file.a;
        let op2 = insn.stack.pop().unwrap();
        let res = acc.bitor(op2);
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, false);

        // Finish
        None
    }
}

/// Pop from stack.
pub mod pop {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0xc1 | 0xd1 | 0xe1 | 0xf1 => (),
            _ => panic!("Illegal instruction."),
        }

        // Pop u16
        let mut word = cpu.popword();
        if insn.opcode == 0xf1 {
            word &= 0xfff0; // lower 4 bits of F cannot be changed
        }
        insn.stack.extend(word.to_le_bytes());

        // Proceed
        insn.exec = delay;
        Some(insn)
    }

    pub fn delay(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to pop a u16.

        // Proceed
        insn.exec = done;
        Some(insn)
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform pop
        let word = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        match insn.opcode {
            0xc1 => cpu.file.bc,
            0xd1 => cpu.file.de,
            0xe1 => cpu.file.hl,
            0xf1 => cpu.file.af,
            _ => panic!("Illegal instruction."),
        }
        .store(&mut cpu.file, word);

        // Finish
        None
    }
}

/// Prefix.
pub mod prefix {
    use super::*;

    pub fn start(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0xcb, "Illegal instruction.");

        // Fetch prefix instruction
        let opcode = cpu.fetchbyte();
        let insn = Instruction::prefix(opcode);
        Some(insn)
    }
}

/// Push to stack.
pub mod push {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        let word = match insn.opcode {
            0xc5 => cpu.file.bc,
            0xd5 => cpu.file.de,
            0xe5 => cpu.file.hl,
            0xf5 => cpu.file.af,
            _ => panic!("Illegal instruction."),
        }
        .load(&cpu.file);
        insn.stack.extend(word.to_le_bytes());

        // Proceed
        insn.exec = push;
        Some(insn)
    }

    pub fn push(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform push
        let word = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        cpu.pushword(word);

        // Proceed
        insn.exec = delay;
        Some(insn)
    }

    pub fn delay(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to push a u16.

        // Proceed
        insn.exec = done;
        Some(insn)
    }

    pub fn done(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the delay needed to fetch the next instruction.

        // Finish
        None
    }
}

/// Reset bit.
pub mod res {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x86 | 0x8e | 0x96 | 0x9e | 0xa6 | 0xae | 0xb6 | 0xbe => {
                let op2 = cpu.readbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0x80..=0xbf => {
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op2);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute RES
        let op1 = (insn.opcode & 0x38) >> 3;
        let op2 = insn.stack.pop().unwrap();
        let mask = !(0b1 << op1);
        let res = mask & op2;

        // Check opcode
        match insn.opcode {
            0x86 | 0x8e | 0x96 | 0x9e | 0xa6 | 0xae | 0xb6 | 0xbe => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x80..=0xbf => {
                // Write X
                helpers::set_op8(cpu, insn.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

/// Return from subroutine.
pub mod ret {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Evaluate condition
        let flags = &mut cpu.file.f;
        let cond = match insn.opcode {
            0xc0 => !Flag::Z.get(flags),
            0xc8 => Flag::Z.get(flags),
            0xc9 => true,
            0xd0 => !Flag::C.get(flags),
            0xd8 => Flag::C.get(flags),
            _ => panic!("Illegal instruction."),
        };
        insn.stack.push(cond as u8);

        if insn.opcode == 0xc9 {
            // Continue
            check(insn, cpu)
        } else {
            // Proceed
            insn.exec = check;
            Some(insn)
        }
    }

    pub fn check(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Execute RET
        let cond = insn.stack.pop().unwrap() != 0;
        if cond {
            // Proceed
            insn.exec = pop;
            Some(insn)
        } else {
            // Finish
            None
        }
    }

    pub fn pop(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Pop PC
        let pc = cpu.popword();
        insn.stack.extend(pc.to_le_bytes());

        // Proceed
        insn.exec = delay;
        Some(insn)
    }

    pub fn delay(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to pop a u16.

        // Proceed
        insn.exec = jump;
        Some(insn)
    }

    pub fn jump(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let pc = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        *cpu.file.pc = pc;

        // Finish
        None
    }
}

/// Return from interrupt service routine.
pub mod reti {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0xd9, "Illegal instruction.");

        // Pop PC
        let pc = cpu.popword();
        insn.stack.extend(pc.to_le_bytes());

        // Proceed
        insn.exec = delay;
        Some(insn)
    }

    pub fn delay(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to pop a u16.

        // Proceed
        insn.exec = jump;
        Some(insn)
    }

    pub fn jump(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let pc = u16::from_le_bytes(
            insn.stack
                .drain(0..=1)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        );
        *cpu.file.pc = pc;

        // Proceed
        insn.exec = done;
        Some(insn)
    }

    pub fn done(_: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Enable interrupts
        cpu.ime = Ime::WillEnable;

        // Finish
        None
    }
}

/// Rotate left (9-bit).
pub mod rl {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x16 => {
                let op1 = cpu.readbyte();
                insn.stack.push(op1);
                insn.exec = done;
                Some(insn)
            }
            0x10..=0x17 => {
                let op1 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op1);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute RL
        let op1 = insn.stack.pop().unwrap();
        let flags = &mut cpu.file.f;
        let cin = Flag::C.get(flags);
        let carry = op1 & 0x80 != 0;
        let res = op1 << 1 | (cin as u8);

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match insn.opcode {
            0x16 => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x10..=0x17 => {
                // Write X
                helpers::set_op8(cpu, insn.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

/// Accumulator rotate left (9-bit).
pub mod rla {
    use super::*;

    pub fn start(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0x17, "Illegal instruction.");

        // Execute RLA
        let flags = &mut cpu.file.f;
        let cin = Flag::C.get(flags);
        let carry = *cpu.file.a & 0x80 != 0;
        let res = *cpu.file.a << 1 | (cin as u8);
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, false);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

/// Rotate left (8-bit).
pub mod rlc {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x06 => {
                let op1 = cpu.readbyte();
                insn.stack.push(op1);
                insn.exec = done;
                Some(insn)
            }
            0x00..=0x07 => {
                let op1 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op1);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute RLC
        let op1 = insn.stack.pop().unwrap();
        let carry = op1 & 0x80 != 0;
        let res = op1 << 1 | (carry as u8);

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match insn.opcode {
            0x06 => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x00..=0x07 => {
                // Write X
                helpers::set_op8(cpu, insn.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

/// Accumulator rotate left (8-bit).
pub mod rlca {
    use super::*;

    pub fn start(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0x07, "Illegal instruction.");

        // Execute RLCA
        let carry = *cpu.file.a & 0x80 != 0;
        let res = *cpu.file.a << 1 | (carry as u8);
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, false);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

/// Rotate right (9-bit).
pub mod rr {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x1e => {
                let op1 = cpu.readbyte();
                insn.stack.push(op1);
                insn.exec = done;
                Some(insn)
            }
            0x18..=0x1f => {
                let op1 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op1);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute RR
        let op1 = insn.stack.pop().unwrap();
        let flags = &mut cpu.file.f;
        let cin = Flag::C.get(flags);
        let carry = op1 & 0x01 != 0;
        let res = ((cin as u8) << 7) | op1 >> 1;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match insn.opcode {
            0x1e => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x18..=0x1f => {
                // Write X
                helpers::set_op8(cpu, insn.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

/// Arithmetic rotate right (9-bit).
pub mod rra {
    use super::*;

    pub fn start(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0x1f, "Illegal instruction.");

        // Execute RRA
        let flags = &mut cpu.file.f;
        let cin = Flag::C.get(flags);
        let carry = *cpu.file.a & 0x01 != 0;
        let res = ((cin as u8) << 7) | *cpu.file.a >> 1;
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, false);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

/// Rotate right (8-bit).
pub mod rrc {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x0e => {
                let op1 = cpu.readbyte();
                insn.stack.push(op1);
                insn.exec = done;
                Some(insn)
            }
            0x08..=0x0f => {
                let op1 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op1);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute RRC
        let op1 = insn.stack.pop().unwrap();
        let carry = op1 & 0x01 != 0;
        let res = ((carry as u8) << 7) | op1 >> 1;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match insn.opcode {
            0x0e => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x08..=0x0f => {
                // Write X
                helpers::set_op8(cpu, insn.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

/// Arithmetic rotate right (8-bit).
pub mod rrca {
    use super::*;

    pub fn start(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0x0f, "Illegal instruction.");

        // Execute RRCA
        let carry = *cpu.file.a & 0x01 != 0;
        let res = ((carry as u8) << 7) | *cpu.file.a >> 1;
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, false);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

/// Reset subroutine.
pub mod rst {
    use super::*;

    pub fn start(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => (),
            _ => panic!("Illegal instruction."),
        };

        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 1 cycle to decrement SP.

        // Proceed
        insn.exec = push;
        Some(insn)
    }

    pub fn push(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Push SP
        cpu.pushword(*cpu.file.pc);

        // Proceed
        insn.exec = delay;
        Some(insn)
    }

    pub fn delay(mut insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to push a u16.

        // Proceed
        insn.exec = jump;
        Some(insn)
    }

    pub fn jump(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Perform jump
        let op1 = match insn.opcode {
            0xc7 => 0x00,
            0xcf => 0x08,
            0xd7 => 0x10,
            0xdf => 0x18,
            0xe7 => 0x20,
            0xef => 0x28,
            0xf7 => 0x30,
            0xff => 0x38,
            _ => panic!("Illegal instruction."),
        };
        *cpu.file.pc = op1;

        // Finish
        None
    }
}

/// Arithmetic subtract with carry.
pub mod sbc {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x9e => {
                let op2 = cpu.readbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xde => {
                let op2 = cpu.fetchbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0x98..=0x9f => {
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op2);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SUB
        let acc = *cpu.file.a;
        let op2 = insn.stack.pop().unwrap();
        let cin = Flag::C.get(&cpu.file.f) as u8;
        let (res, carry0) = acc.overflowing_sub(op2);
        let (res, carry1) = res.overflowing_sub(cin);
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, true);
        Flag::H.set(flags, (op2 & 0x0f) + cin > (acc & 0x0f));
        Flag::C.set(flags, carry0 | carry1);

        // Finish
        None
    }
}

/// Set carry flag.
pub mod scf {
    use super::*;

    pub fn start(insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0x37, "Illegal instruction.");

        // Execute SCF
        let flags = &mut cpu.file.f;
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, true);

        // Finish
        None
    }
}

/// Set bit.
pub mod set {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0xc6 | 0xce | 0xd6 | 0xde | 0xe6 | 0xee | 0xf6 | 0xfe => {
                let op2 = cpu.readbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xc0..=0xff => {
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op2);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SET
        let op1 = (insn.opcode & 0x38) >> 3;
        let op2 = insn.stack.pop().unwrap();
        let mask = !(0b1 << op1);
        let res = (mask & op2) | !mask;

        // Check opcode
        match insn.opcode {
            0xc6 | 0xce | 0xd6 | 0xde | 0xe6 | 0xee | 0xf6 | 0xfe => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0xc0..=0xff => {
                // Write X
                helpers::set_op8(cpu, insn.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

/// Arithmetic shift left.
pub mod sla {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x26 => {
                let op1 = cpu.readbyte();
                insn.stack.push(op1);
                insn.exec = done;
                Some(insn)
            }
            0x20..=0x27 => {
                let op1 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op1);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SLA
        let op1 = insn.stack.pop().unwrap();
        let carry = op1 & 0x80 != 0;
        let res = op1 << 1;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match insn.opcode {
            0x26 => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x20..=0x27 => {
                // Write X
                helpers::set_op8(cpu, insn.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

/// Arithmetic shift right.
pub mod sra {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x2e => {
                let op1 = cpu.readbyte();
                insn.stack.push(op1);
                insn.exec = done;
                Some(insn)
            }
            0x28..=0x2f => {
                let op1 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op1);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SRA
        let op1 = insn.stack.pop().unwrap();
        let sign = op1 & 0x80;
        let carry = op1 & 0x01 != 0;
        let res = sign | (op1 >> 1);

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match insn.opcode {
            0x2e => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x28..=0x2f => {
                // Write X
                helpers::set_op8(cpu, insn.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

/// Logical shift right.
pub mod srl {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x3e => {
                let op1 = cpu.readbyte();
                insn.stack.push(op1);
                insn.exec = done;
                Some(insn)
            }
            0x38..=0x3f => {
                let op1 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op1);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SRL
        let op1 = insn.stack.pop().unwrap();
        let carry = op1 & 0x01 != 0;
        let res = 0x7f & (op1 >> 1);

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, carry);

        // Check opcode
        match insn.opcode {
            0x3e => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x38..=0x3f => {
                // Write X
                helpers::set_op8(cpu, insn.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

/// Stop CPU.
pub mod stop {
    use super::*;

    #[allow(unreachable_code)]
    pub fn start(insn: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        assert!(insn.opcode == 0x10, "Illegal instruction.");

        // Execute STOP
        // <https://gbdev.io/pandocs/imgs/gb_stop.png>
        #[cfg(debug_assertions)]
        todo!("implement this mess of an instruction");

        // Finish
        None
    }
}

/// Arithmetic subtract.
pub mod sub {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x96 => {
                let op2 = cpu.readbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xd6 => {
                let op2 = cpu.fetchbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0x90..=0x97 => {
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op2);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SUB
        let acc = *cpu.file.a;
        let op2 = insn.stack.pop().unwrap();
        let (res, carry) = acc.overflowing_sub(op2);
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, true);
        Flag::H.set(flags, (op2 & 0x0f) > (acc & 0x0f));
        Flag::C.set(flags, carry);

        // Finish
        None
    }
}

/// Swap nibbles.
pub mod swap {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0x36 => {
                let op1 = cpu.readbyte();
                insn.stack.push(op1);
                insn.exec = done;
                Some(insn)
            }
            0x30..=0x37 => {
                let op1 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op1);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute SWAP
        let op1 = insn.stack.pop().unwrap();
        let res = ((op1 & 0xf0) >> 4) | ((op1 & 0x0f) << 4);

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, false);

        // Check opcode
        match insn.opcode {
            0x36 => {
                // Write (HL)
                cpu.writebyte(res);
                // Proceed
                insn.exec = delay;
                Some(insn)
            }
            0x30..=0x37 => {
                // Write X
                helpers::set_op8(cpu, insn.opcode & 0x07, res);
                // Finish
                None
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn delay(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        // Delay by 1 cycle
        // NOTE: This represents the fact that it takes 2 cycles to write a u16.

        // Finish
        None
    }
}

/// Unused instruction.
pub mod unused {
    use super::*;

    pub fn start(_: Instruction, _: &mut Cpu) -> Option<Instruction> {
        panic!("Illegal instruction.");
    }
}

/// Logical XOR.
pub mod xor {
    use super::*;

    pub fn start(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Check opcode
        match insn.opcode {
            0xae => {
                let op2 = cpu.readbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xee => {
                let op2 = cpu.fetchbyte();
                insn.stack.push(op2);
                insn.exec = done;
                Some(insn)
            }
            0xa8..=0xaf => {
                let op2 = helpers::get_op8(cpu, insn.opcode & 0x07);
                insn.stack.push(op2);
                done(insn, cpu)
            }
            _ => panic!("Illegal instruction."),
        }
    }

    pub fn done(mut insn: Instruction, cpu: &mut Cpu) -> Option<Instruction> {
        // Execute XOR
        let acc = *cpu.file.a;
        let op2 = insn.stack.pop().unwrap();
        let res = acc.bitxor(op2);
        *cpu.file.a = res;

        // Set flags
        let flags = &mut cpu.file.f;
        Flag::Z.set(flags, res == 0);
        Flag::N.set(flags, false);
        Flag::H.set(flags, false);
        Flag::C.set(flags, false);

        // Finish
        None
    }
}
