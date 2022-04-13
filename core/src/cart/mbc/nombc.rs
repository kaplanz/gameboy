use std::cell::RefCell;
use std::rc::Rc;

use log::warn;
use remus::{Block, Device};

use super::Mbc;

#[derive(Debug)]
pub struct NoMbc {
    rom: Rc<RefCell<ReadOnly>>,
    ram: Rc<RefCell<dyn Device>>,
}

impl NoMbc {
    pub fn with(rom: Rc<RefCell<dyn Device>>, ram: Rc<RefCell<dyn Device>>) -> Self {
        Self {
            rom: Rc::new(RefCell::new(ReadOnly::from(rom))),
            ram,
        }
    }
}

impl Block for NoMbc {
    fn reset(&mut self) {
        // Reset RAM
        self.ram.borrow_mut().reset();
    }
}

impl Mbc for NoMbc {
    fn rom(&self) -> Rc<RefCell<dyn Device>> {
        self.rom.clone()
    }

    fn ram(&self) -> Rc<RefCell<dyn Device>> {
        self.ram.clone()
    }
}

#[derive(Debug)]
struct ReadOnly(Rc<RefCell<dyn Device>>);

impl Block for ReadOnly {}

impl Device for ReadOnly {
    fn contains(&self, index: usize) -> bool {
        self.0.borrow().contains(index)
    }

    fn len(&self) -> usize {
        self.0.borrow().len()
    }

    fn read(&self, index: usize) -> u8 {
        self.0.borrow().read(index)
    }

    fn write(&mut self, index: usize, value: u8) {
        warn!("called `Device::write({index:#06x}, {value:#04x})` on a `ReadOnly`");
    }
}

impl From<Rc<RefCell<dyn Device>>> for ReadOnly {
    fn from(dev: Rc<RefCell<dyn Device>>) -> Self {
        Self(dev)
    }
}
