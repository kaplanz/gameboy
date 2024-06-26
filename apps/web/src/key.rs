use log::debug;
use rugby::core::dmg::Button;
use rugby::emu::part::joypad::State;
use rugby::prelude::*;
use wasm_bindgen::prelude::*;

use crate::emu::GameBoy;

#[rustfmt::skip]
fn keymap(key: &str) -> Option<Button> {
    match key {
        "x"          => Some(Button::A),
        "z"          => Some(Button::B),
        " "          => Some(Button::Select),
        "Enter"      => Some(Button::Start),
        "ArrowRight" => Some(Button::Right),
        "ArrowLeft"  => Some(Button::Left),
        "ArrowUp"    => Some(Button::Up),
        "ArrowDown"  => Some(Button::Down),
        _ => {
            debug!("unknown key: {key}");
            None
        }
    }
}

#[wasm_bindgen]
impl GameBoy {
    pub fn keydown(&mut self, key: &str) {
        let event = keymap(key).map(|key| (key, State::Dn).into());
        self.0.inside_mut().joypad().recv(event);
    }

    pub fn keyup(&mut self, key: &str) {
        let event = keymap(key).map(|key| (key, State::Up).into());
        self.0.inside_mut().joypad().recv(event);
    }
}
