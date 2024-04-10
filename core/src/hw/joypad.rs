//! Joypad controller.

use std::collections::HashSet;
use std::ops::{BitOr, Not};

use log::{debug, trace};
use remus::bus::Mux;
use remus::dev::Device;
use remus::{Address, Block, Board, Cell, Linked, Shared};

use super::pic::{Interrupt, Pic};
use crate::api::joypad::{Event, Input, Joypad as Api, State};
use crate::dev::Bus;

/// Joypad buttons.
#[rustfmt::skip]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Button {
    A      = 0b0010_0001,
    B      = 0b0010_0010,
    Select = 0b0010_0100,
    Start  = 0b0010_1000,
    Right  = 0b0001_0001,
    Left   = 0b0001_0010,
    Up     = 0b0001_0100,
    Down   = 0b0001_1000,
}

impl Button {
    /// Gets the button's underlying bitmask.
    fn mask(self) -> u8 {
        self as u8
    }
}

impl Input for Button {}

/// Joypad select.
#[derive(Clone, Copy, Debug, Default)]
pub enum Select {
    #[default]
    None = 0b0000_0000,
    DPad = 0b0001_1111,
    Keys = 0b0010_1111,
    Both = 0b0011_1111,
}

impl Select {
    /// Gets the selection's underlying bitmask.
    fn mask(self) -> u8 {
        self as u8
    }
}

/// Joypad model.
#[derive(Debug)]
pub struct Joypad {
    // Control
    // ┌──────┬────────┬─────┬───────┐
    // │ Size │  Name  │ Dev │ Alias │
    // ├──────┼────────┼─────┼───────┤
    // │  1 B │ Joypad │ Reg │ CON   │
    // └──────┴────────┴─────┴───────┘
    con: Shared<Control>,
    // Shared
    pic: Shared<Pic>,
}

impl Joypad {
    /// Constructs a new `Joypad`.
    #[must_use]
    pub fn new(pic: Shared<Pic>) -> Self {
        Self {
            // Control
            con: Shared::default(),
            // Shared
            pic,
        }
    }

    /// Gets a reference to the joypad's control register.
    #[must_use]
    pub fn con(&self) -> Shared<Control> {
        self.con.clone()
    }
}

impl Api for Joypad {
    type Button = Button;

    fn recv(&mut self, events: impl IntoIterator<Item = Event<Self::Button>>) {
        // Borrow controller state
        let keys = &mut self.con.borrow_mut().keys;
        // Update pressed keys
        let mut updated = false;
        for Event { input: key, state } in events {
            trace!("event: {key:?}, {state:?}");
            // Update internal state
            updated |= match state {
                State::Dn => keys.insert(key),
                State::Up => keys.remove(&key),
            };
        }
        // Handle key updates
        if updated {
            debug!("updated keys: {keys:?}");
            // Schedule an interrupt on changes
            self.pic.borrow_mut().req(Interrupt::Joypad);
        } else {
            trace!("received no input events");
        }
    }
}

impl Block for Joypad {
    fn reset(&mut self) {
        // Control
        self.con.reset();
    }
}

impl Board<u16, u8> for Joypad {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let con = self.con().to_dynamic();

        // Map devices on bus          // ┌──────┬──────┬────────┬─────┐
                                       // │ Addr │ Size │  Name  │ Dev │
                                       // ├──────┼──────┼────────┼─────┤
        bus.map(0xff00..=0xff00, con); // │ ff00 │  1 B │ Joypad │ Reg │
                                       // └──────┴──────┴────────┴─────┘
    }
}

impl Linked<Pic> for Joypad {
    fn mine(&self) -> Shared<Pic> {
        self.pic.clone()
    }

    fn link(&mut self, it: Shared<Pic>) {
        self.pic = it;
    }
}

/// Player input register.
#[derive(Debug, Default)]
pub struct Control {
    ctrl: Select,
    keys: HashSet<Button>,
}

impl Address<u16, u8> for Control {
    fn read(&self, _: u16) -> u8 {
        self.load()
    }

    fn write(&mut self, _: u16, value: u8) {
        self.store(value);
    }
}

impl Block for Control {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Cell<u8> for Control {
    fn load(&self) -> u8 {
        // Extract control mode, mask from control
        let mode = 0x30 & self.ctrl.mask();
        let mask = 0x0f & self.ctrl.mask();
        // Determine byte-value from state
        self.keys
            .iter()
            .copied()
            // get bitmask for each key
            .map(Button::mask)
            // keep only selected keys
            .filter(|key| key & mode != 0)
            // mask key index bits
            .map(|key| key & mask)
            // apply selected mode
            .fold(mode, BitOr::bitor)
            // values are read inverted
            .not()
    }

    fn store(&mut self, value: u8) {
        // NOTE: Only key select bits are writable
        const MASK: u8 = 0b0011_0000;
        // Update selection control
        self.ctrl = match !value & MASK {
            0b0000_0000 => Select::None,
            0b0001_0000 => Select::DPad,
            0b0010_0000 => Select::Keys,
            0b0011_0000 => Select::Both,
            _ => unreachable!(),
        };
    }
}

impl Device<u16, u8> for Control {}
