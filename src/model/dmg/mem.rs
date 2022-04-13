use std::cell::RefCell;
use std::rc::Rc;

use remus::mem::Ram;
use remus::Block;

use super::boot;

#[derive(Debug, Default)]
pub struct Memory {
    // ┌────────┬──────┬─────┬───────┐
    // │  SIZE  │ NAME │ DEV │ ALIAS │
    // ├────────┼──────┼─────┼───────┤
    // │  256 B │ Boot │ ROM │       │
    // │ 8 Ki B │ Work │ RAM │ WRAM  │
    // │  127 B │ High │ RAM │ HRAM  │
    // └────────┴──────┴─────┴───────┘
    pub boot: Rc<RefCell<boot::Rom>>,
    pub wram: Rc<RefCell<Ram<0x2000>>>,
    pub hram: Rc<RefCell<Ram<0x007f>>>,
}

impl Block for Memory {
    fn reset(&mut self) {
        // Reset boot ROM
        self.boot.borrow_mut().reset();
    }
}
