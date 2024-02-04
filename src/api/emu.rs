//! Emulator API.

/// Emulator interface.
pub trait Emulator {
    /// A type specifying [`Emulator`] errors.
    type Error;

    /// Initializes a newly instantiated emulator.
    ///
    /// # Errors
    ///
    /// Errors if initialization failed.
    fn init(&mut self) -> Result<(), Self::Error>;

    /// Loads a cartridge ROM into the emulator.
    ///
    /// # Errors
    ///
    /// Errors if the cartridge could not be loaded.
    fn load(&mut self, rom: &[u8]) -> Result<(), Self::Error>;

    /// Ticks a single cycle of the emulator.
    fn tick(&mut self);

    /// Fast-forwards any number of cycles.
    fn ffwd(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.tick();
        }
    }
}
