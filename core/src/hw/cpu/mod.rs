//! CPU models.
//!
//! The following CPU models may be used within an emulator. To provide a
//! unified interface, all models implement the [`Cpu`] trait.

use remus::{Block, Machine};

pub mod sm83;

pub use self::sm83::Cpu as Sm83;

/// Unified processor interface.
pub trait Model: Block + Machine {
    /// The processor's instruction set.
    type Instruction;

    /// Gets the current instruction.
    fn insn(&self) -> Self::Instruction;

    /// Move the PC to the provided address.
    fn goto(&mut self, pc: u16);

    /// Execute the provided instruction in-place.
    fn exec(&mut self, opcode: u8);

    /// Run the provided program (i.e. instruction sequence) in-place.
    fn run(&mut self, prog: &[u8]);

    /// Enable (or wake) the processor.
    fn wake(&mut self);
}
