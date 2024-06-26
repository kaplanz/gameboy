use wasm_bindgen::prelude::*;

pub mod emu;
pub mod key;

#[wasm_bindgen(start)]
fn start() {
    console_log::init().expect("error initializing logger");
}
