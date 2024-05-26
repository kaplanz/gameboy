//! Processor API.

use rugby_arch::{Byte, Word};

/// Processor support.
pub trait Support {
    /// Compute interface.
    type Proc: Processor;

    /// Gets the core's processor.
    #[must_use]
    fn cpu(&self) -> &Self::Proc;

    /// Mutably gets the core's processor.
    #[must_use]
    fn cpu_mut(&mut self) -> &mut Self::Proc;
}

/// Processor interface.
pub trait Processor {
    /// Instruction Set Architecture (ISA).
    ///
    /// Represents a valid instance of an instruction in the processor's ISA.
    type Insn;

    /// Gets the current instruction.
    fn insn(&self) -> Self::Insn;

    /// Move the PC to the provided address.
    fn goto(&mut self, pc: Word);

    /// Execute the provided instruction in-place.
    fn exec(&mut self, code: Byte);

    /// Run the provided program (i.e. instruction sequence) in-place.
    fn run(&mut self, prog: &[Byte]);

    /// Enable (or wake) the processor.
    fn wake(&mut self);
}
