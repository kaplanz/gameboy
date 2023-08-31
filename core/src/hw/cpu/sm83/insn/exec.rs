//! Instruction execution implementations.
//!
//! For more information, see the [instruction set][optable].
//!
//! # Legend
//!
//! - `n8` means immediate 8-bit data.
//! - `n16` means immediate little-endian 16-bit data.
//! - `a8` means 8-bit unsigned data, which is added to 0xFF00 in certain
//!   instructions to create a 16-bit address in HRAM (High RAM).
//! - `a16` means little-endian 16-bit address.
//! - `e8` means 8-bit signed data.
//!
//! [optable]: https://gbdev.io/gb-opcodes/optables/

#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::wildcard_imports)]

use super::{helpers, Cpu, Error, Execute, Flag, Ime, Instruction, Result, Status};

type Return = Result<Option<Operation>>;

/// Instruction operation state.
#[derive(Clone, Debug)]
pub enum Operation {
    Adc(adc::Adc),
    Add(add::Add),
    Addw(addw::Addw),
    And(and::And),
    Bit(bit::Bit),
    Call(call::Call),
    Ccf(ccf::Ccf),
    Cp(cp::Cp),
    Cpl(cpl::Cpl),
    Daa(daa::Daa),
    Dec(dec::Dec),
    Decw(decw::Decw),
    Di(di::Di),
    Ei(ei::Ei),
    Halt(halt::Halt),
    Inc(inc::Inc),
    Incw(incw::Incw),
    Int(int::Int),
    Jp(jp::Jp),
    Jr(jr::Jr),
    Ld(ld::Ld),
    Ldh(ldh::Ldh),
    Ldw(ldw::Ldw),
    Nop(nop::Nop),
    Or(or::Or),
    Pop(pop::Pop),
    Prefix(prefix::Prefix),
    Push(push::Push),
    Res(res::Res),
    Ret(ret::Ret),
    Reti(reti::Reti),
    Rl(rl::Rl),
    Rla(rla::Rla),
    Rlc(rlc::Rlc),
    Rlca(rlca::Rlca),
    Rr(rr::Rr),
    Rra(rra::Rra),
    Rrc(rrc::Rrc),
    Rrca(rrca::Rrca),
    Rst(rst::Rst),
    Sbc(sbc::Sbc),
    Scf(scf::Scf),
    Set(set::Set),
    Sla(sla::Sla),
    Sra(sra::Sra),
    Srl(srl::Srl),
    Stop(stop::Stop),
    Sub(sub::Sub),
    Swap(swap::Swap),
    Unused(unused::Unused),
    Xor(xor::Xor),
}

impl Execute for Operation {
    #[rustfmt::skip]
    fn exec(self, code: u8, cpu: &mut Cpu) -> Result<Option<Operation>> {
        match self {
            Operation::Adc(inner)    => inner.exec(code, cpu),
            Operation::Add(inner)    => inner.exec(code, cpu),
            Operation::Addw(inner)   => inner.exec(code, cpu),
            Operation::And(inner)    => inner.exec(code, cpu),
            Operation::Bit(inner)    => inner.exec(code, cpu),
            Operation::Call(inner)   => inner.exec(code, cpu),
            Operation::Ccf(inner)    => inner.exec(code, cpu),
            Operation::Cp(inner)     => inner.exec(code, cpu),
            Operation::Cpl(inner)    => inner.exec(code, cpu),
            Operation::Daa(inner)    => inner.exec(code, cpu),
            Operation::Dec(inner)    => inner.exec(code, cpu),
            Operation::Decw(inner)   => inner.exec(code, cpu),
            Operation::Di(inner)     => inner.exec(code, cpu),
            Operation::Ei(inner)     => inner.exec(code, cpu),
            Operation::Halt(inner)   => inner.exec(code, cpu),
            Operation::Inc(inner)    => inner.exec(code, cpu),
            Operation::Incw(inner)   => inner.exec(code, cpu),
            Operation::Int(inner)    => inner.exec(code, cpu),
            Operation::Jp(inner)     => inner.exec(code, cpu),
            Operation::Jr(inner)     => inner.exec(code, cpu),
            Operation::Ld(inner)     => inner.exec(code, cpu),
            Operation::Ldh(inner)    => inner.exec(code, cpu),
            Operation::Ldw(inner)    => inner.exec(code, cpu),
            Operation::Nop(inner)    => inner.exec(code, cpu),
            Operation::Or(inner)     => inner.exec(code, cpu),
            Operation::Pop(inner)    => inner.exec(code, cpu),
            Operation::Prefix(inner) => inner.exec(code, cpu),
            Operation::Push(inner)   => inner.exec(code, cpu),
            Operation::Res(inner)    => inner.exec(code, cpu),
            Operation::Ret(inner)    => inner.exec(code, cpu),
            Operation::Reti(inner)   => inner.exec(code, cpu),
            Operation::Rl(inner)     => inner.exec(code, cpu),
            Operation::Rla(inner)    => inner.exec(code, cpu),
            Operation::Rlc(inner)    => inner.exec(code, cpu),
            Operation::Rlca(inner)   => inner.exec(code, cpu),
            Operation::Rr(inner)     => inner.exec(code, cpu),
            Operation::Rra(inner)    => inner.exec(code, cpu),
            Operation::Rrc(inner)    => inner.exec(code, cpu),
            Operation::Rrca(inner)   => inner.exec(code, cpu),
            Operation::Rst(inner)    => inner.exec(code, cpu),
            Operation::Sbc(inner)    => inner.exec(code, cpu),
            Operation::Scf(inner)    => inner.exec(code, cpu),
            Operation::Set(inner)    => inner.exec(code, cpu),
            Operation::Sla(inner)    => inner.exec(code, cpu),
            Operation::Sra(inner)    => inner.exec(code, cpu),
            Operation::Srl(inner)    => inner.exec(code, cpu),
            Operation::Stop(inner)   => inner.exec(code, cpu),
            Operation::Sub(inner)    => inner.exec(code, cpu),
            Operation::Swap(inner)   => inner.exec(code, cpu),
            Operation::Unused(inner) => inner.exec(code, cpu),
            Operation::Xor(inner)    => inner.exec(code, cpu),
        }
    }
}

/// Arithmetic add with carry.
pub(super) mod adc;

/// Arithmetic add.
pub(super) mod add;

/// Arithmetic add wide (16-bit).
pub(super) mod addw;

/// Logical AND.
pub(super) mod and;

/// Test bit.
pub(super) mod bit;

/// Call subroutine.
pub(super) mod call;

/// Complement carry flag.
pub(super) mod ccf;

/// Compare.
pub(super) mod cp;

/// Complement.
pub(super) mod cpl;

/// Decimal adjust after addition.
pub(super) mod daa;

/// Decrement.
pub(super) mod dec;

/// Decrement wide (16-bit).
pub(super) mod decw;

/// Disable interrupts.
pub(super) mod di;

/// Enable interrupts.
pub(super) mod ei;

/// Halt CPU.
pub(super) mod halt;

/// Increment.
pub(super) mod inc;

/// Increment wide (16-bit).
pub(super) mod incw;

/// Interrupt service routine.
pub(super) mod int;

/// Jump.
pub(super) mod jp;

/// Jump relative.
pub(super) mod jr;

/// Load.
pub(super) mod ld;

/// Load wide (16-bit).
pub(super) mod ldw;

/// Load high.
pub(super) mod ldh;

/// No operation.
pub(super) mod nop;

/// Logical OR.
pub(super) mod or;

/// Pop from stack.
pub(super) mod pop;

/// Prefix.
pub(super) mod prefix;

/// Push to stack.
pub(super) mod push;

/// Reset bit.
pub(super) mod res;

/// Return from subroutine.
pub(super) mod ret;

/// Return from interrupt service routine.
pub(super) mod reti;

/// Rotate left (9-bit).
pub(super) mod rl;

/// Accumulator rotate left (9-bit).
pub(super) mod rla;

/// Rotate left (8-bit).
pub(super) mod rlc;

/// Accumulator rotate left (8-bit).
pub(super) mod rlca;

/// Rotate right (9-bit).
pub(super) mod rr;

/// Arithmetic rotate right (9-bit).
pub(super) mod rra;

/// Rotate right (8-bit).
pub(super) mod rrc;

/// Arithmetic rotate right (8-bit).
pub(super) mod rrca;

/// Reset subroutine.
pub(super) mod rst;

/// Arithmetic subtract with carry.
pub(super) mod sbc;

/// Set carry flag.
pub(super) mod scf;

/// Set bit.
pub(super) mod set;

/// Arithmetic shift left.
pub(super) mod sla;

/// Arithmetic shift right.
pub(super) mod sra;

/// Logical shift right.
pub(super) mod srl;

/// Stop CPU.
pub(super) mod stop;

/// Arithmetic subtract.
pub(super) mod sub;

/// Swap nibbles.
pub(super) mod swap;

/// Unused instruction.
pub(super) mod unused;

/// Logical XOR.
pub(super) mod xor;
