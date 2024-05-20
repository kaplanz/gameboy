//! Hardware timer.

use log::{debug, trace};
use remus::mio::{Bus, Mmio};
use remus::reg::{Port, Register};
use remus::{Block, Byte, Shared};

use super::pic::{self, Interrupt};

/// Timer register select.
///
/// See more details [here][regs].
///
/// [regs]: https://gbdev.io/pandocs/Timer_and_Divider_Registers.html
#[derive(Clone, Copy, Debug)]
pub enum Select {
    /// `[$FF04]`: Divider register.
    Div,
    /// `[$FF05]`: Timer counter.
    Tima,
    /// `[$FF06]`: Timer modulo.
    Tma,
    /// `[$FF07]`: Timer control.
    Tac,
}

/// Hardware timer.
#[derive(Debug)]
pub struct Timer {
    /// Timer registers.
    pub reg: Control,
    /// Timer internals.
    etc: Internal,
    /// Interrupt line.
    int: pic::Line,
}

/// Timer internals.
#[derive(Debug, Default)]
struct Internal {
    /// Previous AND result.
    and: bool,
}

impl Internal {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Timer {
    /// Constructs a new `Timer`.
    #[must_use]
    pub fn new(int: pic::Line) -> Self {
        Self {
            reg: Control::default(),
            etc: Internal::default(),
            int,
        }
    }

    /// Calculates the AND result.
    ///
    /// Used to determine whether TIMA will be incremented, as documented by
    /// [Hacktix][gbedg].
    ///
    /// [gbedg]: https://github.com/Hacktix/GBEDG/blob/master/timers/index.md#timer-operation
    fn andres(&self) -> bool {
        let ena = self.reg.tac.borrow().enable();
        let sel = self.reg.tac.borrow().select();
        let div = self.reg.div.borrow().div();
        ena && (sel & div != 0)
    }
}

impl Block for Timer {
    fn ready(&self) -> bool {
        true
    }

    #[rustfmt::skip]
    fn cycle(&mut self) {
        // Increment the divider every T-cycle.
        //
        // Since only the upper 8-bits of DIV are mapped, has the observable
        // effect of incrementing DIV (as read by the CPU) every 256 T-cycles.
        self.reg.div.borrow_mut().inc();

        // Reload TIMA
        let reload = self.reg.tima.borrow().rel == reg::Reload::Now;
        self.reg.tima.borrow_mut().rel.tick();
        if reload {
            // Reload from TMA
            let tma = self.reg.tma.load();
            self.reg.tima.store(tma);
            debug!("timer reloaded");
            // Request an interrupt
            self.int.raise(Interrupt::Timer);
        }

        // Check if TIMA should be incremented
        let this = self.andres();      // calculate AND result
        let tick = self.etc.and && !this; // check for falling edge
        self.etc.and = this;              // store for next cycle

        // Increment TIMA
        if tick {
            let carry = self.reg.tima.borrow_mut().inc();
            trace!("timer: {}", self.reg.tima.load());
            // Trigger pending reload on overflow
            if carry {
                self.reg.tima.borrow_mut().rel.sched();
                debug!("timer reload pending");
            }
        }
    }

    fn reset(&mut self) {
        self.reg.reset();
        self.etc.reset();
    }
}

impl Mmio for Timer {
    fn attach(&self, bus: &mut Bus) {
        self.reg.attach(bus);
    }
}

impl Port<Byte> for Timer {
    type Select = Select;

    fn load(&self, reg: Self::Select) -> Byte {
        match reg {
            Select::Div => self.reg.div.load(),
            Select::Tima => self.reg.tima.load(),
            Select::Tma => self.reg.tma.load(),
            Select::Tac => self.reg.tac.load(),
        }
    }

    fn store(&mut self, reg: Self::Select, value: Byte) {
        match reg {
            Select::Div => self.reg.div.store(value),
            Select::Tima => self.reg.tima.store(value),
            Select::Tma => self.reg.tma.store(value),
            Select::Tac => self.reg.tac.store(value),
        }
    }
}

/// Timer registers.
///
/// | Address | Size | Name | Description      |
/// |:-------:|------|------|------------------|
/// | `$FF04` | Byte | DIV  | Divider register |
/// | `$FF05` | Byte | TIMA | Timer counter    |
/// | `$FF06` | Byte | TMA  | Timer modulo     |
/// | `$FF07` | Byte | TAC  | Timer control    |
#[derive(Debug, Default)]
pub struct Control {
    /// Divider register.
    pub div: Shared<reg::Div>,
    /// Timer counter.
    pub tima: Shared<reg::Tima>,
    /// Timer modulo.
    pub tma: Shared<reg::Tma>,
    /// Timer control.
    pub tac: Shared<reg::Tac>,
}

impl Block for Control {
    fn reset(&mut self) {
        self.div.take();
        self.tima.take();
        self.tma.take();
        self.tac.take();
    }
}

impl Mmio for Control {
    fn attach(&self, bus: &mut Bus) {
        bus.map(0xff04..=0xff04, self.div.clone().into());
        bus.map(0xff05..=0xff05, self.tima.clone().into());
        bus.map(0xff06..=0xff06, self.tma.clone().into());
        bus.map(0xff07..=0xff07, self.tac.clone().into());
    }
}

/// Timer register models.
pub mod reg {
    use log::debug;
    use remus::mem::Memory;
    use remus::reg::Register;
    use remus::{Byte, Word};

    /// Divider register.
    #[derive(Debug, Default)]
    pub struct Div(Word);

    impl Div {
        /// Gets the internal clock (lower 8-bits).
        #[allow(unused)]
        #[must_use]
        pub(super) fn clk(&self) -> Byte {
            self.0.load().to_le_bytes()[0]
        }

        /// Gets the full internal register value.
        #[must_use]
        pub(super) fn div(&self) -> Word {
            self.0.load()
        }

        /// Increment the divider register.
        pub(super) fn inc(&mut self) {
            let value = self.0.load().wrapping_add(1);
            self.0.store(value);
        }
    }

    impl Memory for Div {
        fn read(&self, _: Word) -> remus::mem::Result<Byte> {
            Ok(self.load())
        }

        fn write(&mut self, _: Word, data: Byte) -> remus::mem::Result<()> {
            self.store(data);
            Ok(())
        }
    }

    impl Register for Div {
        type Value = Byte;

        /// Loads the value of DIV (upper 8-bits).
        fn load(&self) -> Self::Value {
            self.0.load().to_le_bytes()[1]
        }

        fn store(&mut self, _: Self::Value) {
            debug!("resetting divider");
            self.0.store(0);
        }
    }

    /// Timer counter.
    #[derive(Debug, Default)]
    pub struct Tima {
        pub(super) reg: Byte,
        pub(super) rel: Reload,
    }

    /// Timer reload counter.
    ///
    /// In effect, this models the 1 M-cycle (4 T-cycle) delay between a reload
    /// being triggered and it occurring.
    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    pub(super) enum Reload {
        /// Timer is not amid a reload.
        #[default]
        None,
        /// Reload pending in `N` cycles.
        Wait(u8),
        /// Reload occurring this cycle.
        Now,
    }

    impl Reload {
        /// Set a reload to occur.
        pub(super) fn sched(&mut self) {
            assert!(matches!(self, Self::None));
            *self = Self::Wait(2);
        }

        /// Advance the reload counter.
        pub(super) fn tick(&mut self) {
            *self = match self {
                Reload::Wait(0) => Reload::Now,
                // Decrement cycles until reload
                Reload::Wait(mut count) => {
                    // Decrement cycles until reload
                    count -= 1;
                    // Update the reload counter
                    Reload::Wait(count)
                }
                // Reload just occurred, or counter is inactive
                _ => Reload::None,
            }
        }
    }

    impl Tima {
        /// Increment the timer counter.
        #[must_use]
        pub(super) fn inc(&mut self) -> bool {
            let (value, carry) = self.reg.load().overflowing_add(1);
            self.reg.store(value);
            carry
        }
    }

    impl Memory for Tima {
        fn read(&self, _: Word) -> remus::mem::Result<Byte> {
            Ok(self.load())
        }

        fn write(&mut self, _: Word, data: Byte) -> remus::mem::Result<()> {
            self.store(data);
            Ok(())
        }
    }

    impl Register for Tima {
        type Value = Byte;

        fn load(&self) -> Self::Value {
            self.reg.load()
        }

        fn store(&mut self, value: Self::Value) {
            // Ignore stores to TIMA right before a reload occurs
            if self.rel != Reload::Now {
                self.rel = Reload::None;
                self.reg.store(value);
            }
        }
    }

    /// Timer modulo.
    pub type Tma = Byte;

    /// Timer control.
    #[derive(Debug, Default)]
    pub struct Tac(Byte);

    impl Tac {
        /// Gets the enable bit.
        #[must_use]
        pub fn enable(&self) -> bool {
            self.0.load() & 0b100 != 0
        }

        /// Gets the clock select rate.
        #[must_use]
        pub fn select(&self) -> Word {
            match self.0.load() & 0b011 {
                0b01 => 1 << 3,
                0b10 => 1 << 5,
                0b11 => 1 << 7,
                0b00 => 1 << 9,
                _ => unreachable!(),
            }
        }
    }

    impl Memory for Tac {
        fn read(&self, _: Word) -> remus::mem::Result<Byte> {
            Ok(self.load())
        }

        fn write(&mut self, _: Word, data: Byte) -> remus::mem::Result<()> {
            self.store(data);
            Ok(())
        }
    }

    impl Register for Tac {
        type Value = Byte;

        fn load(&self) -> Self::Value {
            0b1111_1000 | self.0.load()
        }

        fn store(&mut self, value: Self::Value) {
            self.0.store(value & 0b111);
        }
    }
}

#[allow(clippy::too_many_lines)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tima_reload_works() {
        // Configure 65536 Hz timer (64 cycles)
        let mut timer = Timer::new(pic::Pic::new().line);
        timer.reg.tac.store(0b110);
        timer.reg.tma.store(0xfe);
        timer.reg.tima.store(0xfe);

        for _ in 0..64 {
            assert_eq!(timer.reg.tima.load(), 0xfe);
            timer.cycle();
        } // increment -> 0xff
        for _ in 0..64 {
            assert_eq!(timer.reg.tima.load(), 0xff);
            timer.cycle();
        } // overflow  -> 0x00
        for _ in 0..4 {
            assert_eq!(timer.reg.tima.load(), 0x00);
            timer.cycle();
        } // reload    -> 0xfe
        for _ in 4..64 {
            assert_eq!(timer.reg.tima.load(), 0xfe);
            timer.cycle();
        } // increment -> 0xff
        for _ in 0..64 {
            assert_eq!(timer.reg.tima.load(), 0xff);
            timer.cycle();
        } // overflow  -> 0x00
        for _ in 0..4 {
            assert_eq!(timer.reg.tima.load(), 0x00);
            timer.cycle();
        } // reload    -> 0xfe
        for _ in 4..64 {
            assert_eq!(timer.reg.tima.load(), 0xfe);
            timer.cycle();
        } // increment -> 0xff
        assert_eq!(timer.reg.tima.load(), 0xff);
    }

    #[test]
    fn tima_write_reloading_working() {
        let line = pic::Pic::new().line;
        // Test 1
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(line.clone());
            timer.reg.tac.store(0b110);
            timer.reg.tma.store(0xfe);
            timer.reg.tima.store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            timer.reg.tima.store(0xfd);
            //   overwrite -> 0xfd
            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xfd);
                timer.cycle();
            } // increment -> 0xfe
            assert_eq!(timer.reg.tima.load(), 0xfe);
        }

        // Test 2
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(line.clone());
            timer.reg.tac.store(0b110);
            timer.reg.tma.store(0xfe);
            timer.reg.tima.store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..1 {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            }
            timer.reg.tima.store(0xfd);
            //   overwrite -> 0xfd
            for _ in 1..64 {
                assert_eq!(timer.reg.tima.load(), 0xfd);
                timer.cycle();
            } // increment -> 0xfe
            assert_eq!(timer.reg.tima.load(), 0xfe);
        }

        // Test 3
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(line.clone());
            timer.reg.tac.store(0b110);
            timer.reg.tma.store(0xfe);
            timer.reg.tima.store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..2 {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            }
            timer.reg.tima.store(0xfd);
            //   overwrite -> 0xfd
            for _ in 2..64 {
                assert_eq!(timer.reg.tima.load(), 0xfd);
                timer.cycle();
            } // increment -> 0xfe
            assert_eq!(timer.reg.tima.load(), 0xfe);
        }

        // Test 4
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(line.clone());
            timer.reg.tac.store(0b110);
            timer.reg.tma.store(0xfe);
            timer.reg.tima.store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..3 {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            } // reloading now
            timer.reg.tima.store(0xfd); // write ignored!
            {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            } // reload    -> 0xfe
            for _ in 4..64 {
                assert_eq!(timer.reg.tima.load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            assert_eq!(timer.reg.tima.load(), 0xff);
        }

        // Test 5
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(line.clone());
            timer.reg.tac.store(0b110);
            timer.reg.tma.store(0xfe);
            timer.reg.tima.store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..4 {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            } // reload    -> 0xfe
            timer.reg.tima.store(0xfd);
            // overwrite   -> 0xfd
            for _ in 4..64 {
                assert_eq!(timer.reg.tima.load(), 0xfd);
                timer.cycle();
            } // increment -> 0xfe
            assert_eq!(timer.reg.tima.load(), 0xfe);
        }
    }

    #[test]
    fn tma_write_reloading_working() {
        let line = pic::Pic::new().line;
        // Test 1
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(line.clone());
            timer.reg.tac.store(0b110);
            timer.reg.tma.store(0xfe);
            timer.reg.tima.store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            timer.reg.tma.store(0x69);
            for _ in 0..4 {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            } // reload    -> 0x69
            for _ in 4..64 {
                assert_eq!(timer.reg.tima.load(), 0x69);
                timer.cycle();
            } // increment -> 0x6a
            assert_eq!(timer.reg.tima.load(), 0x6a);
        }

        // Test 2
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(line.clone());
            timer.reg.tac.store(0b110);
            timer.reg.tma.store(0xfe);
            timer.reg.tima.store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..1 {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            }
            timer.reg.tma.store(0x69);
            for _ in 1..4 {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            } // reload    -> 0x69
            for _ in 4..64 {
                assert_eq!(timer.reg.tima.load(), 0x69);
                timer.cycle();
            } // increment -> 0x6a
            assert_eq!(timer.reg.tima.load(), 0x6a);
        }

        // Test 3
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(line.clone());
            timer.reg.tac.store(0b110);
            timer.reg.tma.store(0xfe);
            timer.reg.tima.store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..2 {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            }
            timer.reg.tma.store(0x69);
            for _ in 2..4 {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            } // reload    -> 0x69
            for _ in 4..64 {
                assert_eq!(timer.reg.tima.load(), 0x69);
                timer.cycle();
            } // increment -> 0x6a
            assert_eq!(timer.reg.tima.load(), 0x6a);
        }

        // Test 4
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(line.clone());
            timer.reg.tac.store(0b110);
            timer.reg.tma.store(0xfe);
            timer.reg.tima.store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..3 {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            }
            timer.reg.tma.store(0x69);
            for _ in 3..4 {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            } // reload    -> 0x69
            for _ in 4..64 {
                assert_eq!(timer.reg.tima.load(), 0x69);
                timer.cycle();
            } // increment -> 0x6a
            assert_eq!(timer.reg.tima.load(), 0x6a);
        }

        // Test 5
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(line.clone());
            timer.reg.tac.store(0b110);
            timer.reg.tma.store(0xfe);
            timer.reg.tima.store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.reg.tima.load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..4 {
                assert_eq!(timer.reg.tima.load(), 0x00);
                timer.cycle();
            } // reload    -> 0xfe
            timer.reg.tma.store(0x69); // too late!
            for _ in 4..64 {
                assert_eq!(timer.reg.tima.load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            assert_eq!(timer.reg.tima.load(), 0xff);
        }
    }
}
