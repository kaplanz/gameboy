//! Audio model.

use remus::mem::Ram;
use remus::mio::{Bus, Mmio};
use remus::reg::{Port, Register};
use remus::{Block, Byte, Shared};

use crate::api::audio::Audio as Api;

/// Wave RAM.
///
/// 16 byte RAM used to store audio samples. See more details [here][wave].
///
/// [wave]: https://gbdev.io/pandocs/Audio_Registers.html#ff30ff3f--wave-pattern-ram
pub type Wave = Ram<[Byte; 0x0010]>;

/// Audio register select.
///
/// See more details [here][regs].
///
/// [regs]: https://gbdev.io/pandocs/Audio_Registers.html
#[derive(Clone, Copy, Debug)]
pub enum Select {
    /// `[$FF26]`: Audio master control
    Nr52,
    /// `[$FF25]`: Sound panning
    Nr51,
    /// `[$FF24]`: Master volume & VIN panning
    Nr50,
    /// `[$FF10]`: CH1 period sweep
    Nr10,
    /// `[$FF11]`: CH1 length timer & duty cycle
    Nr11,
    /// `[$FF12]`: CH1 volume & envelope
    Nr12,
    /// `[$FF13]`: CH1 period low
    Nr13,
    /// `[$FF14]`: CH1 period high & control
    Nr14,
    /// `[$FF16]`: CH2 length timer & duty cycle
    Nr21,
    /// `[$FF17]`: CH2 volume & envelope
    Nr22,
    /// `[$FF18]`: CH2 period low
    Nr23,
    /// `[$FF19]`: CH2 period high & control
    Nr24,
    /// `[$FF1A]`: CH3 DAC enable
    Nr30,
    /// `[$FF1B]`: CH3 length timer
    Nr31,
    /// `[$FF1C]`: CH3 output level
    Nr32,
    /// `[$FF1D]`: CH3 period low
    Nr33,
    /// `[$FF1E]`: CH3 period high & control
    Nr34,
    /// `[$FF20]`: CH4 length timer
    Nr41,
    /// `[$FF21]`: CH4 volume & envelope
    Nr42,
    /// `[$FF22]`: CH4 frequency & randomness
    Nr43,
    /// `[$FF23]`: CH4 control
    Nr44,
}

/// Audio processing unit.
#[derive(Debug, Default)]
pub struct Apu {
    /// Audio registers.
    pub reg: Control,
    /// Audio memory.
    pub mem: Bank,
}

impl Apu {
    /// Constructs a new `Apu`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Api for Apu {}

impl Block for Apu {
    fn ready(&self) -> bool {
        true
    }

    fn cycle(&mut self) {
        todo!()
    }

    fn reset(&mut self) {
        self.reg.reset();
    }
}

impl Mmio for Apu {
    fn attach(&self, bus: &mut Bus) {
        self.reg.attach(bus);
        self.mem.attach(bus);
    }
}

impl Port<Byte> for Apu {
    type Select = Select;

    fn load(&self, reg: Self::Select) -> Byte {
        match reg {
            Select::Nr52 => self.reg.nr52.load(),
            Select::Nr51 => self.reg.nr51.load(),
            Select::Nr50 => self.reg.nr50.load(),
            Select::Nr10 => self.reg.nr10.load(),
            Select::Nr11 => self.reg.nr11.load(),
            Select::Nr12 => self.reg.nr12.load(),
            Select::Nr13 => self.reg.nr13.load(),
            Select::Nr14 => self.reg.nr14.load(),
            Select::Nr21 => self.reg.nr21.load(),
            Select::Nr22 => self.reg.nr22.load(),
            Select::Nr23 => self.reg.nr23.load(),
            Select::Nr24 => self.reg.nr24.load(),
            Select::Nr30 => self.reg.nr30.load(),
            Select::Nr31 => self.reg.nr31.load(),
            Select::Nr32 => self.reg.nr32.load(),
            Select::Nr33 => self.reg.nr33.load(),
            Select::Nr34 => self.reg.nr34.load(),
            Select::Nr41 => self.reg.nr41.load(),
            Select::Nr42 => self.reg.nr42.load(),
            Select::Nr43 => self.reg.nr43.load(),
            Select::Nr44 => self.reg.nr44.load(),
        }
    }

    fn store(&mut self, reg: Self::Select, value: Byte) {
        match reg {
            Select::Nr52 => self.reg.nr52.store(value),
            Select::Nr51 => self.reg.nr51.store(value),
            Select::Nr50 => self.reg.nr50.store(value),
            Select::Nr10 => self.reg.nr10.store(value),
            Select::Nr11 => self.reg.nr11.store(value),
            Select::Nr12 => self.reg.nr12.store(value),
            Select::Nr13 => self.reg.nr13.store(value),
            Select::Nr14 => self.reg.nr14.store(value),
            Select::Nr21 => self.reg.nr21.store(value),
            Select::Nr22 => self.reg.nr22.store(value),
            Select::Nr23 => self.reg.nr23.store(value),
            Select::Nr24 => self.reg.nr24.store(value),
            Select::Nr30 => self.reg.nr30.store(value),
            Select::Nr31 => self.reg.nr31.store(value),
            Select::Nr32 => self.reg.nr32.store(value),
            Select::Nr33 => self.reg.nr33.store(value),
            Select::Nr34 => self.reg.nr34.store(value),
            Select::Nr41 => self.reg.nr41.store(value),
            Select::Nr42 => self.reg.nr42.store(value),
            Select::Nr43 => self.reg.nr43.store(value),
            Select::Nr44 => self.reg.nr44.store(value),
        }
    }
}

/// Audio registers.
///
/// | Address | Size | Name | Description                   |
/// |:-------:|------|------|-------------------------------|
/// | `$FF26` | Byte | NR52 | Audio master control          |
/// | `$FF25` | Byte | NR51 | Sound panning                 |
/// | `$FF24` | Byte | NR50 | Master volume & VIN panning   |
/// | `$FF10` | Byte | NR10 | CH1 period sweep              |
/// | `$FF11` | Byte | NR11 | CH1 length timer & duty cycle |
/// | `$FF12` | Byte | NR12 | CH1 volume & envelope         |
/// | `$FF13` | Byte | NR13 | CH1 period low                |
/// | `$FF14` | Byte | NR14 | CH1 period high & control     |
/// | `$FF16` | Byte | NR21 | CH2 length timer & duty cycle |
/// | `$FF17` | Byte | NR22 | CH2 volume & envelope         |
/// | `$FF18` | Byte | NR23 | CH2 period low                |
/// | `$FF19` | Byte | NR24 | CH2 period high & control     |
/// | `$FF1A` | Byte | NR30 | CH3 DAC enable                |
/// | `$FF1B` | Byte | NR31 | CH3 length timer              |
/// | `$FF1C` | Byte | NR32 | CH3 output level              |
/// | `$FF1D` | Byte | NR33 | CH3 period low                |
/// | `$FF1E` | Byte | NR34 | CH3 period high & control     |
/// | `$FF20` | Byte | NR41 | CH4 length timer              |
/// | `$FF21` | Byte | NR42 | CH4 volume & envelope         |
/// | `$FF22` | Byte | NR43 | CH4 frequency & randomness    |
/// | `$FF23` | Byte | NR44 | CH4 control                   |
///
/// [regs]: https://gbdev.io/pandocs/Audio_Registers.html
#[derive(Debug, Default)]
pub struct Control {
    /// Audio master control.
    pub nr52: Shared<Byte>,
    /// Sound panning.
    pub nr51: Shared<Byte>,
    /// Master volume & VIN panning.
    pub nr50: Shared<Byte>,
    /// CH1 period sweep.
    pub nr10: Shared<Byte>,
    /// CH1 length timer & duty cycle.
    pub nr11: Shared<Byte>,
    /// CH1 volume & envelope.
    pub nr12: Shared<Byte>,
    /// CH1 period low.
    pub nr13: Shared<Byte>,
    /// CH1 period high & control.
    pub nr14: Shared<Byte>,
    /// CH2 length timer & duty cycle.
    pub nr21: Shared<Byte>,
    /// CH2 volume & envelope.
    pub nr22: Shared<Byte>,
    /// CH2 period low.
    pub nr23: Shared<Byte>,
    /// CH2 period high & control.
    pub nr24: Shared<Byte>,
    /// CH3 DAC enable.
    pub nr30: Shared<Byte>,
    /// CH3 length timer.
    pub nr31: Shared<Byte>,
    /// CH3 output level.
    pub nr32: Shared<Byte>,
    /// CH3 period low.
    pub nr33: Shared<Byte>,
    /// CH3 period high & control.
    pub nr34: Shared<Byte>,
    /// CH4 length timer.
    pub nr41: Shared<Byte>,
    /// CH4 volume & envelope.
    pub nr42: Shared<Byte>,
    /// CH4 frequency & randomness.
    pub nr43: Shared<Byte>,
    /// CH4 control.
    pub nr44: Shared<Byte>,
}

impl Block for Control {
    fn reset(&mut self) {
        // Global Control Registers
        self.nr52.take();
        self.nr51.take();
        self.nr50.take();
        // Sound Channel 1 — Pulse with wavelength sweep
        self.nr10.take();
        self.nr11.take();
        self.nr12.take();
        self.nr13.take();
        self.nr14.take();
        // Sound Channel 2 — Pulse
        self.nr21.take();
        self.nr22.take();
        self.nr23.take();
        self.nr24.take();
        // Sound Channel 3 — Wave output
        self.nr30.take();
        self.nr31.take();
        self.nr32.take();
        self.nr33.take();
        self.nr34.take();
        // Sound Channel 4 — Noise
        self.nr41.take();
        self.nr42.take();
        self.nr43.take();
        self.nr44.take();
    }
}

impl Mmio for Control {
    fn attach(&self, bus: &mut Bus) {
        // Global Control Registers
        bus.map(0xff26..=0xff26, self.nr52.clone().into());
        bus.map(0xff25..=0xff25, self.nr51.clone().into());
        bus.map(0xff24..=0xff24, self.nr50.clone().into());
        // Sound Channel 1 — Pulse with wavelength sweep
        bus.map(0xff10..=0xff10, self.nr10.clone().into());
        bus.map(0xff11..=0xff11, self.nr11.clone().into());
        bus.map(0xff12..=0xff12, self.nr12.clone().into());
        bus.map(0xff13..=0xff13, self.nr13.clone().into());
        bus.map(0xff14..=0xff14, self.nr14.clone().into());
        // Sound Channel 2 — Pulse
        bus.map(0xff16..=0xff16, self.nr21.clone().into());
        bus.map(0xff17..=0xff17, self.nr22.clone().into());
        bus.map(0xff18..=0xff18, self.nr23.clone().into());
        bus.map(0xff19..=0xff19, self.nr24.clone().into());
        // Sound Channel 3 — Wave output
        bus.map(0xff1a..=0xff1a, self.nr30.clone().into());
        bus.map(0xff1b..=0xff1b, self.nr31.clone().into());
        bus.map(0xff1c..=0xff1c, self.nr32.clone().into());
        bus.map(0xff1d..=0xff1d, self.nr33.clone().into());
        bus.map(0xff1e..=0xff1e, self.nr34.clone().into());
        // Sound Channel 4 — Noise
        bus.map(0xff20..=0xff20, self.nr41.clone().into());
        bus.map(0xff21..=0xff21, self.nr42.clone().into());
        bus.map(0xff22..=0xff22, self.nr43.clone().into());
        bus.map(0xff23..=0xff23, self.nr44.clone().into());
    }
}

/// Audio memory.
///
/// |     Address     | Size | Name | Description |
/// |:---------------:|------|------|-------------|
/// | `$FF30..=$FF3F` | 16 B | WAVE | Wave RAM    |
#[derive(Debug, Default)]
pub struct Bank {
    /// Wave RAM.
    pub wave: Shared<Wave>,
}

impl Block for Bank {}

impl Mmio for Bank {
    fn attach(&self, bus: &mut Bus) {
        bus.map(0xff30..=0xff40, self.wave.clone().into());
    }
}
