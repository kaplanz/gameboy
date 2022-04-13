use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use remus::{Block, Device};

pub use self::mbc1::Mbc1;
pub use self::nombc::NoMbc;

mod mbc1;
mod nombc;

pub trait Mbc: Block + Debug {
    fn rom(&self) -> Rc<RefCell<dyn Device>>;

    fn ram(&self) -> Rc<RefCell<dyn Device>>;
}
