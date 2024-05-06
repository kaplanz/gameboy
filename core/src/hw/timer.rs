//! Hardware timer.

use log::{debug, trace};
use remus::bus::Mux;
use remus::dev::Device;
use remus::reg::Register;
use remus::{Address, Block, Board, Cell, Linked, Location, Machine, Shared};

use super::pic::{Interrupt, Pic};
use crate::dev::Bus;

/// 8-bit timer control register set.
///
/// For more info, see [here][regs].
///
/// [regs]: https://gbdev.io/pandocs/Timer_and_Divider_Registers.html
#[derive(Clone, Copy, Debug)]
pub enum Control {
    /// `0xFF04`: Divider register.
    Div,
    /// `0xFF05`: Timer counter.
    Tima,
    /// `0xFF06`: Timer modulo.
    Tma,
    /// `0xFF07`: Timer control.
    Tac,
}

/// Timer model.
#[rustfmt::skip]
#[derive(Debug)]
pub struct Timer {
    // State
    prev: bool,
    // Control
    // ┌──────┬──────────┬─────┐
    // │ Size │   Name   │ Dev │
    // ├──────┼──────────┼─────┤
    // │  4 B │ Control  │ Reg │
    // └──────┴──────────┴─────┘
    file: File,
    // Shared
    pic: Shared<Pic>,
}

impl Timer {
    /// Constructs a new `Timer`.
    #[must_use]
    pub fn new(pic: Shared<Pic>) -> Self {
        Self {
            // State
            prev: bool::default(),
            // Control
            file: File::default(),
            // Shared
            pic,
        }
    }

    /// Gets a reference to the timer's divider register.
    #[must_use]
    pub fn div(&self) -> Shared<Div> {
        self.file.div.clone()
    }

    /// Gets a reference to the timer's counter register.
    #[must_use]
    pub fn tima(&self) -> Shared<Tima> {
        self.file.tima.clone()
    }

    /// Gets a reference to the timer's modulo register.
    #[must_use]
    pub fn tma(&self) -> Shared<Tma> {
        self.file.tma.clone()
    }

    /// Gets a reference to the timer's control register.
    #[must_use]
    pub fn tac(&self) -> Shared<Tac> {
        self.file.tac.clone()
    }

    /// Calculates the AND result.
    ///
    /// Used to determine whether TIMA will be incremented, as documented by
    /// [Hacktix][gbedg].
    ///
    /// [gbedg]: https://github.com/Hacktix/GBEDG/blob/master/timers/index.md#timer-operation
    fn andres(&self) -> bool {
        let ena = self.file.tac.borrow().ena();
        let sel = self.file.tac.borrow().sel();
        let div = self.file.div.borrow().div();
        ena && (sel & div != 0)
    }
}

impl Block for Timer {
    fn reset(&mut self) {
        // State
        std::mem::take(&mut self.prev);
        // Control
        self.file.reset();
    }
}

impl Board<u16, u8> for Timer {
    fn connect(&self, bus: &mut Bus) {
        // Control
        self.file.connect(bus);
    }
}

impl Linked<Pic> for Timer {
    fn mine(&self) -> Shared<Pic> {
        self.pic.clone()
    }

    fn link(&mut self, it: Shared<Pic>) {
        self.pic = it;
    }
}

#[rustfmt::skip]
impl Location<u8> for Timer {
    type Register = Control;

    fn load(&self, reg: Self::Register) -> u8 {
        match reg {
            Control::Div  => self.file.div.load(),
            Control::Tima => self.file.tima.load(),
            Control::Tma  => self.file.tma.load(),
            Control::Tac  => self.file.tac.load(),
        }
    }

    fn store(&mut self, reg: Self::Register, value: u8) {
        match reg {
            Control::Div  => self.file.div.store(value),
            Control::Tima => self.file.tima.store(value),
            Control::Tma  => self.file.tma.store(value),
            Control::Tac  => self.file.tac.store(value),
        }
    }
}

impl Machine for Timer {
    fn enabled(&self) -> bool {
        true
    }

    #[rustfmt::skip]
    fn cycle(&mut self) {
        // Increment the divider every T-cycle.
        //
        // Since only the upper 8-bits of DIV are mapped, has the observable
        // effect of incrementing DIV (as read by the CPU) every 256 T-cycles.
        self.file.div.borrow_mut().inc();

        // Reload TIMA
        let reload = self.file.tima.borrow().rel == Reload::Active;
        self.file.tima.borrow_mut().rel.tick();
        if reload {
            // Reload from TMA
            let tma = self.file.tma.load();
            self.file.tima.store(tma);
            debug!("timer reloaded");
            // Request an interrupt
            self.pic.borrow_mut().req(Interrupt::Timer);
        }

        // Check if TIMA should be incremented
        let this = self.andres();      // calculate AND result
        let tick = self.prev && !this; // check for falling edge
        self.prev = this;              // store for next cycle

        // Increment TIMA
        if tick {
            let carry = self.file.tima.borrow_mut().inc();
            trace!("timer: {}", self.file.tima.load());
            // Trigger pending reload on overflow
            if carry {
                self.file.tima.borrow_mut().rel.sched();
                debug!("timer reload pending");
            }
        }
    }
}

/// Control registers.
#[rustfmt::skip]
#[derive(Debug, Default)]
struct File {
    // ┌──────┬──────────┬─────┬───────┐
    // │ Size │   Name   │ Dev │ Alias │
    // ├──────┼──────────┼─────┼───────┤
    // │  1 B │ Divider  │ Reg │ DIV   │
    // │  1 B │ Counter  │ Reg │ TIMA  │
    // │  1 B │ Modulo   │ Reg │ TMA   │
    // │  1 B │ Control  │ Reg │ TAC   │
    // └──────┴──────────┴─────┴───────┘
    div:  Shared<Div>,
    tima: Shared<Tima>,
    tma:  Shared<Tma>,
    tac:  Shared<Tac>,
}

impl Block for File {
    fn reset(&mut self) {
        self.div.reset();
        self.tima.reset();
        self.tma.reset();
        self.tac.reset();
    }
}

impl Board<u16, u8> for File {
    #[rustfmt::skip]
    fn connect(&self, bus: &mut Bus) {
        // Extract devices
        let div  = self.div.clone().to_dynamic();
        let tima = self.tima.clone().to_dynamic();
        let tma  = self.tma.clone().to_dynamic();
        let tac  = self.tac.clone().to_dynamic();

        // Map devices on bus           // ┌──────┬──────┬──────────┬─────┐
                                        // │ Addr │ Size │   Name   │ Dev │
                                        // ├──────┼──────┼──────────┼─────┤
        bus.map(0xff04..=0xff04, div);  // │ ff04 │  1 B │ Divider  │ Reg │
        bus.map(0xff05..=0xff05, tima); // │ ff05 │  1 B │ Counter  │ Reg │
        bus.map(0xff06..=0xff06, tma);  // │ ff06 │  1 B │ Modulo   │ Reg │
        bus.map(0xff07..=0xff07, tac);  // │ ff07 │  1 B │ Control  │ Reg │
                                        // └──────┴──────┴──────────┴─────┘
    }
}

/// Divider register.
#[derive(Debug, Default)]
pub struct Div(Register<u16>);

impl Div {
    /// Gets the internal clock (lower 8-bits).
    #[must_use]
    pub fn clk(&self) -> u8 {
        self.0.load().to_le_bytes()[0]
    }

    /// Gets the full internal register value.
    #[must_use]
    pub fn div(&self) -> u16 {
        self.0.load()
    }

    /// Increment the divider register.
    fn inc(&mut self) {
        let value = self.0.load().wrapping_add(1);
        self.0.store(value);
    }
}

impl Address<u16, u8> for Div {
    fn read(&self, _: u16) -> u8 {
        self.load()
    }

    fn write(&mut self, _: u16, value: u8) {
        self.store(value);
    }
}

impl Block for Div {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Cell<u8> for Div {
    /// Loads the value of DIV (upper 8-bits).
    fn load(&self) -> u8 {
        self.0.load().to_le_bytes()[1]
    }

    fn store(&mut self, _: u8) {
        debug!("resetting divider");
        self.0.store(0);
    }
}

impl Device<u16, u8> for Div {}

/// Timer counter.
#[derive(Debug, Default)]
pub struct Tima {
    reg: Register<u8>,
    rel: Reload,
}

/// Timer reload counter.
///
/// In effect, this models the 1 M-cycle (4 T-cycle) delay between a reload
/// being triggered and it occuring.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum Reload {
    /// Timer is not amid a reload.
    #[default]
    Inactive,
    /// Reload pending in `N` cycles.
    Pending(u8),
    /// Reload occuring this cycle.
    Active,
}

impl Reload {
    /// Set a reload to occur.
    fn sched(&mut self) {
        assert!(matches!(self, Self::Inactive));
        *self = Self::Pending(2);
    }

    /// Advance the reload counter.
    fn tick(&mut self) {
        *self = match self {
            Reload::Pending(0) => Reload::Active,
            // Decrement cycles until reload
            Reload::Pending(mut count) => {
                // Decrement cycles until reload
                count -= 1;
                // Update the reload counter
                Reload::Pending(count)
            }
            // Reload just occured, or counter is inactive
            _ => Reload::Inactive,
        }
    }
}

impl Tima {
    /// Increment the timer counter.
    #[must_use]
    fn inc(&mut self) -> bool {
        let (value, carry) = self.reg.load().overflowing_add(1);
        self.reg.store(value);
        carry
    }
}

impl Address<u16, u8> for Tima {
    fn read(&self, _: u16) -> u8 {
        self.load()
    }

    fn write(&mut self, _: u16, value: u8) {
        self.store(value);
    }
}

impl Block for Tima {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Cell<u8> for Tima {
    fn load(&self) -> u8 {
        self.reg.load()
    }

    fn store(&mut self, value: u8) {
        // Ignore stores to TIMA right before a reload occurs
        if self.rel != Reload::Active {
            self.rel = Reload::Inactive;
            self.reg.store(value);
        }
    }
}

impl Device<u16, u8> for Tima {}

/// Timer modulo.
pub type Tma = Register<u8>;

/// Timer control.
#[derive(Debug, Default)]
pub struct Tac(Register<u8>);

impl Tac {
    /// Gets the enable bit.
    #[must_use]
    pub fn ena(&self) -> bool {
        self.0.load() & 0b100 != 0
    }

    /// Gets the clock select rate.
    #[must_use]
    pub fn sel(&self) -> u16 {
        match self.0.load() & 0b011 {
            0b01 => 1 << 3,
            0b10 => 1 << 5,
            0b11 => 1 << 7,
            0b00 => 1 << 9,
            _ => unreachable!(),
        }
    }
}

impl Address<u16, u8> for Tac {
    fn read(&self, _: u16) -> u8 {
        self.load()
    }

    fn write(&mut self, _: u16, value: u8) {
        self.store(value);
    }
}

impl Block for Tac {
    fn reset(&mut self) {
        std::mem::take(self);
    }
}

impl Cell<u8> for Tac {
    fn load(&self) -> u8 {
        0b1111_1000 | self.0.load()
    }

    fn store(&mut self, value: u8) {
        self.0.store(value & 0b111);
    }
}

impl Device<u16, u8> for Tac {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tima_reload_works() {
        // Configure 65536 Hz timer (64 cycles)
        let mut timer = Timer::new(Shared::default());
        timer.tac().store(0b110);
        timer.tma().store(0xfe);
        timer.tima().store(0xfe);

        for _ in 0..64 {
            assert_eq!(timer.tima().load(), 0xfe);
            timer.cycle();
        } // increment -> 0xff
        for _ in 0..64 {
            assert_eq!(timer.tima().load(), 0xff);
            timer.cycle();
        } // overflow  -> 0x00
        for _ in 0..4 {
            assert_eq!(timer.tima().load(), 0x00);
            timer.cycle();
        } // reload    -> 0xfe
        for _ in 4..64 {
            assert_eq!(timer.tima().load(), 0xfe);
            timer.cycle();
        } // increment -> 0xff
        for _ in 0..64 {
            assert_eq!(timer.tima().load(), 0xff);
            timer.cycle();
        } // overflow  -> 0x00
        for _ in 0..4 {
            assert_eq!(timer.tima().load(), 0x00);
            timer.cycle();
        } // reload    -> 0xfe
        for _ in 4..64 {
            assert_eq!(timer.tima().load(), 0xfe);
            timer.cycle();
        } // increment -> 0xff
        assert_eq!(timer.tima().load(), 0xff);
    }

    #[test]
    fn tima_write_reloading_working() {
        // Test 1
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(Shared::default());
            timer.tac().store(0b110);
            timer.tma().store(0xfe);
            timer.tima().store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..0 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            }
            timer.tima().store(0xfd);
            //   overwrite -> 0xfd
            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xfd);
                timer.cycle();
            } // increment -> 0xfe
            assert_eq!(timer.tima().load(), 0xfe);
        }

        // Test 2
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(Shared::default());
            timer.tac().store(0b110);
            timer.tma().store(0xfe);
            timer.tima().store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..1 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            }
            timer.tima().store(0xfd);
            //   overwrite -> 0xfd
            for _ in 1..64 {
                assert_eq!(timer.tima().load(), 0xfd);
                timer.cycle();
            } // increment -> 0xfe
            assert_eq!(timer.tima().load(), 0xfe);
        }

        // Test 3
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(Shared::default());
            timer.tac().store(0b110);
            timer.tma().store(0xfe);
            timer.tima().store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..2 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            }
            timer.tima().store(0xfd);
            //   overwrite -> 0xfd
            for _ in 2..64 {
                assert_eq!(timer.tima().load(), 0xfd);
                timer.cycle();
            } // increment -> 0xfe
            assert_eq!(timer.tima().load(), 0xfe);
        }

        // Test 4
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(Shared::default());
            timer.tac().store(0b110);
            timer.tma().store(0xfe);
            timer.tima().store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..3 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            } // reloading now
            timer.tima().store(0xfd); // write ignored!
            {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            } // reload    -> 0xfe
            for _ in 4..64 {
                assert_eq!(timer.tima().load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            assert_eq!(timer.tima().load(), 0xff);
        }

        // Test 5
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(Shared::default());
            timer.tac().store(0b110);
            timer.tma().store(0xfe);
            timer.tima().store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..4 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            } // reload    -> 0xfe
            timer.tima().store(0xfd);
            // overwrite   -> 0xfd
            for _ in 4..64 {
                assert_eq!(timer.tima().load(), 0xfd);
                timer.cycle();
            } // increment -> 0xfe
            assert_eq!(timer.tima().load(), 0xfe);
        }
    }

    #[test]
    fn tma_write_reloading_working() {
        // Test 1
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(Shared::default());
            timer.tac().store(0b110);
            timer.tma().store(0xfe);
            timer.tima().store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..0 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            }
            timer.tma().store(0x69);
            for _ in 0..4 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            } // reload    -> 0x69
            for _ in 4..64 {
                assert_eq!(timer.tima().load(), 0x69);
                timer.cycle();
            } // increment -> 0x6a
            assert_eq!(timer.tima().load(), 0x6a);
        }

        // Test 2
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(Shared::default());
            timer.tac().store(0b110);
            timer.tma().store(0xfe);
            timer.tima().store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..1 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            }
            timer.tma().store(0x69);
            for _ in 1..4 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            } // reload    -> 0x69
            for _ in 4..64 {
                assert_eq!(timer.tima().load(), 0x69);
                timer.cycle();
            } // increment -> 0x6a
            assert_eq!(timer.tima().load(), 0x6a);
        }

        // Test 3
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(Shared::default());
            timer.tac().store(0b110);
            timer.tma().store(0xfe);
            timer.tima().store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..2 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            }
            timer.tma().store(0x69);
            for _ in 2..4 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            } // reload    -> 0x69
            for _ in 4..64 {
                assert_eq!(timer.tima().load(), 0x69);
                timer.cycle();
            } // increment -> 0x6a
            assert_eq!(timer.tima().load(), 0x6a);
        }

        // Test 4
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(Shared::default());
            timer.tac().store(0b110);
            timer.tma().store(0xfe);
            timer.tima().store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..3 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            }
            timer.tma().store(0x69);
            for _ in 3..4 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            } // reload    -> 0x69
            for _ in 4..64 {
                assert_eq!(timer.tima().load(), 0x69);
                timer.cycle();
            } // increment -> 0x6a
            assert_eq!(timer.tima().load(), 0x6a);
        }

        // Test 5
        {
            // Configure 65536 Hz timer (64 cycles)
            let mut timer = Timer::new(Shared::default());
            timer.tac().store(0b110);
            timer.tma().store(0xfe);
            timer.tima().store(0xfe);

            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            for _ in 0..64 {
                assert_eq!(timer.tima().load(), 0xff);
                timer.cycle();
            } // overflow  -> 0x00
            for _ in 0..4 {
                assert_eq!(timer.tima().load(), 0x00);
                timer.cycle();
            } // reload    -> 0xfe
            timer.tma().store(0x69); // too late!
            for _ in 4..64 {
                assert_eq!(timer.tima().load(), 0xfe);
                timer.cycle();
            } // increment -> 0xff
            assert_eq!(timer.tima().load(), 0xff);
        }
    }
}
