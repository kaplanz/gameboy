//! Cartridge API.

/// Cartridge support.
pub trait Support {
    /// Cartridge interface.
    type Cartridge: Cartridge;

    /// Gets the inserted cartridge.
    fn cart(&mut self) -> Option<&Self::Cartridge>;

    /// Mutably gets the inserted cartridge.
    fn cart_mut(&mut self) -> Option<&mut Self::Cartridge>;

    /// Inserts and loads a cartridge.
    fn load(&mut self, cart: Self::Cartridge);

    /// Ejects the inserted cartridge.
    #[must_use]
    fn eject(&mut self) -> Option<Self::Cartridge>;
}

/// Cartridge interface.
pub trait Cartridge {}
