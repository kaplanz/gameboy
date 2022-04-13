use std::ops::Deref;

pub const SCREEN: Resolution = Resolution {
    width: 160,
    height: 144,
};

#[derive(Debug)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

impl Resolution {
    pub const fn len(&self) -> usize {
        self.width.saturating_mul(self.height)
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait Emulator {
    fn send(&mut self, btns: Vec<Button>);

    fn redraw<F>(&self, draw: F)
    where
        F: FnMut(&[u32]);
}

#[rustfmt::skip]
#[derive(Copy, Clone, Debug)]
pub enum Button {
    A      = 0b00100001,
    B      = 0b00100010,
    Select = 0b00100100,
    Start  = 0b00101000,
    Right  = 0b00010001,
    Left   = 0b00010010,
    Up     = 0b00010100,
    Down   = 0b00011000,
}

#[derive(Debug)]
pub struct Screen([u32; SCREEN.len()]);

impl Deref for Screen {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<[u32; SCREEN.len()]> for Screen {
    fn from(buf: [u32; SCREEN.len()]) -> Self {
        Self(buf)
    }
}
