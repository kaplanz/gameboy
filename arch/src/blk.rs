use crate::Shared;

/// Logical emulation block.
pub trait Block {
    /// Check if the block is enabled.
    ///
    /// # Note
    ///
    /// When disabled, this indicates that the block has no work to perform. If
    /// the result is not checked before calling [`tick`](Block::cycle), the
    /// block may behave incorrectly and end up in an undefined state. (This can
    /// always be fixed with a [`reset`](Block::reset)).
    fn ready(&self) -> bool {
        true
    }

    /// Emulates a single cycle of the block.
    ///
    /// # Note
    ///
    /// Before calling this function, it is usually recommended to ensure the
    /// block is [`ready`](Block::ready).
    fn cycle(&mut self) {}

    /// Performs a reset on the block.
    ///
    /// Afterwards, the block should behave as if it has just been
    /// initialized[^1] to its powered-on state.
    ///
    /// [^1]: Models should be aware that sometimes persistent data is left
    ///       behind intentionally by the implementation. Within the context of
    ///       an emulator, accessing persistent data after a reset should be
    ///       considered undefined behaviour.
    fn reset(&mut self) {}
}

impl<B: Block> Block for Shared<B> {
    fn ready(&self) -> bool {
        self.borrow().ready()
    }

    fn cycle(&mut self) {
        self.borrow_mut().cycle();
    }

    fn reset(&mut self) {
        self.borrow_mut().reset();
    }
}
