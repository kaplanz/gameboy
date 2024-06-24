use rugby::arch::Block;
use rugby::core::dmg;
use rugby::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn start() {
    console_log::init().expect("error initializing logger");
}

#[derive(Debug, Default)]
#[wasm_bindgen(inspectable)]
pub struct GameBoy(dmg::GameBoy);

#[wasm_bindgen]
impl GameBoy {
    /// Constructs a new `Emulator`.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(dmg::GameBoy::new())
    }

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

#[wasm_bindgen]
impl GameBoy {
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

#[wasm_bindgen]
impl GameBoy {
    /// Checks if the frame is ready to be rendered.
    pub fn vsync(&self) -> bool {
        self.0.inside().video().vsync()
    }

    pub fn frame(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(
            self.0
                .inside()
                .video()
                .frame()
                .into_iter()
                .map(|&pix| pix as u8)
                .collect::<Box<[u8]>>()
                .as_ref(),
        )
    }
}

#[wasm_bindgen]
pub struct Cartridge(dmg::Cartridge);

#[wasm_bindgen]
impl Cartridge {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(dmg::Cartridge::new(include_bytes!("../../../roms/test/acid2/dmg-acid2.gb")).unwrap())
    }
}
