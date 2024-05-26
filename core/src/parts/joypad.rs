//! Player input device.

use std::collections::HashSet;
use std::ops::{BitOr, Not};

use log::{debug, trace};
use rugby_arch::mem::Memory;
use rugby_arch::mio::{Bus, Mmio};
use rugby_arch::reg::Register;
use rugby_arch::{Block, Byte, Shared, Word};

use super::pic::{self, Interrupt};
use crate::api::joypad::{Event, Input, Joypad as Api, State};

/// Joypad inputs.
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
    fn mode(self) -> Byte {
        Mode::MASK & self as Byte
    }

    fn key(self) -> Byte {
        Mode::KEYS & self as Byte
    }
}

impl Input for Button {}

/// Joypad register mode.
#[derive(Clone, Copy, Debug, Default)]
pub enum Mode {
    /// Select none.
    #[default]
    None = 0b0000_0000,
    /// Select d-pad.
    DPad = 0b0001_1111,
    /// Select button.
    Keys = 0b0010_1111,
    /// Select both.
    Both = 0b0011_1111,
}

impl Mode {
    const KEYS: Byte = 0b0000_1111;
    const MASK: Byte = 0b0011_0000;

    fn mode(self) -> Byte {
        Self::MASK & self as Byte
    }

    fn select(self, btn: Button) -> bool {
        btn.mode() & self.mode() != 0
    }
}

impl From<Byte> for Mode {
    fn from(value: Byte) -> Self {
        match value & Self::MASK {
            0b0000_0000 => Mode::None,
            0b0001_0000 => Mode::DPad,
            0b0010_0000 => Mode::Keys,
            0b0011_0000 => Mode::Both,
            _ => unreachable!(),
        }
    }
}

/// Joypad controller.
#[derive(Debug)]
pub struct Joypad {
    /// Joypad register.
    pub con: Shared<Control>,
    /// Interrupt line.
    int: pic::Line,
}

impl Joypad {
    /// Constructs a new `Joypad`.
    #[must_use]
    pub fn new(int: pic::Line) -> Self {
        Self {
            con: Shared::new(Control::default()),
            int,
        }
    }
}

impl Api for Joypad {
    type Button = Button;

    fn recv(&mut self, events: impl IntoIterator<Item = Event<Self::Button>>) {
        // Borrow controller state
        let mut con = self.con.borrow_mut();
        let mode = con.mode;
        let keys = &mut con.keys;
        // Update pressed keys
        let mut updated = false;
        let mut trigger = false;
        for Event { input: btn, state } in events {
            trace!("event: {btn:?}, {state:?}");
            // Update internal state
            updated |= match state {
                State::Dn => keys.insert(btn),
                State::Up => keys.remove(&btn),
            };
            // Determine if an interrupt should occur
            trigger |= state == State::Dn && mode.select(btn);
        }
        // Handle key updates
        if updated {
            debug!("updated keys: {keys:?}");
            // Schedule an interrupt on changes
            if trigger {
                self.int.raise(Interrupt::Joypad);
            }
        } else {
            trace!("received no input events");
        }
    }
}

impl Block for Joypad {
    fn reset(&mut self) {
        self.con.reset();
    }
}

impl Mmio for Joypad {
    fn attach(&self, bus: &mut Bus) {
        bus.map(0xff00..=0xff00, self.con.clone().into());
    }
}

/// Joypad register.
#[derive(Debug, Default)]
pub struct Control {
    mode: Mode,
    keys: HashSet<Button>,
}

impl Block for Control {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Memory for Control {
    fn read(&self, _: Word) -> rugby_arch::mem::Result<Byte> {
        Ok(self.load())
    }

    fn write(&mut self, _: Word, data: Byte) -> rugby_arch::mem::Result<()> {
        self.store(data);
        Ok(())
    }
}

impl Register for Control {
    type Value = Byte;

    fn load(&self) -> Self::Value {
        self.keys
            .iter()
            .copied()
            // keep only selected keys
            .filter(|btn| self.mode.select(*btn))
            // mask the button's key
            .map(Button::key)
            // apply selected mode
            .fold(self.mode.mode(), BitOr::bitor)
            // values are read inverted
            .not()
    }

    fn store(&mut self, value: Self::Value) {
        self.mode = Mode::from(
            // values are written inverted
            !value,
        );
    }
}
