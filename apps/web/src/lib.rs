use rugby::arch::Block;
use rugby::core::dmg::GameBoy;
use wasm_bindgen::prelude::*;

#[derive(Debug, Default)]
#[wasm_bindgen(inspectable)]
pub struct Emulator(GameBoy);

#[wasm_bindgen]
impl Emulator {
    /// Constructs a new `Emulator`.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(GameBoy::new())
    }
}

#[wasm_bindgen]
impl Emulator {
    /// Checks if enabled.
    pub fn ready(&mut self) -> bool {
        self.0.ready()
    }

    /// Emulates a single cycle.
    pub fn cycle(&mut self) {
        self.0.cycle();
    }

    /// Performs a reset.
    pub fn reset(&mut self) {
        self.0.reset();
    }
}
