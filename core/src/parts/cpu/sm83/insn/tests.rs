use std::iter;

use remus::mio::Bus;

use super::*;
use crate::parts::pic::Pic;

fn setup() -> Cpu {
    let bus = Bus::default();
    Cpu::new(bus, Pic::new().line)
}

#[test]
fn cycle_count() {
    // Test each instruction
    for code in 0..=0xff {
        // Declare expected cycle count
        let expect = CYCLES[code as usize];
        // Create CPU model, decode instruction
        let mut cpu = setup();
        let mut insn = Instruction::new(code);
        // Count instruction execution cycles
        let found = if expect > 0 {
            1 + iter::from_fn(move || {
                insn.clone()
                    .exec(&mut cpu)
                    .unwrap_or_else(|_| panic!("should execute: {code:#04X} ; {insn}"))
                    .map(|next| insn = next)
            })
            .count()
        } else {
            expect
        };
        // Confirm match with cycle table
        assert_eq!(
            expect,
            found,
            "mismatch in cycle count for insn: {code:#04X} ; {insn}",
            insn = Instruction::new(code)
        );
    }
}

#[test]
fn prefix_cycle_count() {
    // Test each prefix instruction
    for code in 0..=0xff {
        // Declare expected cycle count
        let expect = PREFIX[code as usize];
        // Create CPU model, decode prefixed instruction
        let mut cpu = setup();
        let mut insn = Instruction::prefix(code);
        // Count instruction execution cycles
        let found = 2 + iter::from_fn(move || {
            insn.clone()
                .exec(&mut cpu)
                .unwrap_or_else(|_| panic!("should execute: {code:#04X} ; {insn}"))
                .map(|next| insn = next)
        })
        .count();
        // Confirm match with cycle table
        assert_eq!(
            expect,
            found,
            "mismatch in cycle count for insn: {code:#04X} ; {insn}",
            insn = Instruction::prefix(code)
        );
    }
}

const CYCLES: [usize; 0x100] = [
    1, 3, 2, 2, 1, 1, 2, 1, 5, 2, 2, 2, 1, 1, 2, 1, // 0x00
    0, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1, 2, 1, // 0x10
    3, 3, 2, 2, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 0x20
    3, 3, 2, 2, 3, 3, 3, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 0x30
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 0x40
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 0x50
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 0x60
    2, 2, 2, 2, 2, 2, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1, // 0x70
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 0x80
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 0x90
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 0xa0
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 0xb0
    5, 3, 4, 4, 6, 4, 2, 4, 2, 4, 3, 0, 3, 6, 2, 4, // 0xc0
    5, 3, 4, 0, 6, 4, 2, 4, 2, 4, 3, 0, 3, 0, 2, 4, // 0xd0
    3, 3, 2, 0, 0, 4, 2, 4, 4, 1, 4, 0, 0, 0, 2, 4, // 0xe0
    3, 3, 2, 1, 0, 4, 2, 4, 3, 2, 4, 1, 0, 0, 2, 4, // 0xf0
];

const PREFIX: [usize; 0x100] = [
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0x00
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0x10
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0x20
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0x30
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 0x40
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 0x50
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 0x60
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 0x70
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0x80
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0x90
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0xa0
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0xb0
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0xc0
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0xd0
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0xe0
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0xf0
];
