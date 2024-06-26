use rugby::core::dmg;
use wasm_bindgen::prelude::*;

use super::GameBoy;

#[wasm_bindgen]
pub fn acid2() -> Box<[u8]> {
    Box::from(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../roms/test/acid2/dmg-acid2.gb"
        ))
        .as_slice(),
    )
}

#[wasm_bindgen]
pub struct Cartridge(dmg::Cartridge);

#[wasm_bindgen]
impl Cartridge {
    #[wasm_bindgen(constructor)]
    pub fn new(rom: &[u8]) -> Self {
        Self(dmg::Cartridge::new(rom).unwrap())
    }
}

#[wasm_bindgen]
impl GameBoy {
    /// Inserts a game cartridge.
    ///
    /// If a cartridge is already inserted, it will first be
    /// [ejected](Self::eject).
    pub fn insert(&mut self, cart: Cartridge) {
        self.0.insert(cart.0);
    }

    /// Ejects the inserted game cartridge, if any.
    pub fn eject(&mut self) -> Option<Cartridge> {
        self.0.eject().map(Cartridge)
    }
}
