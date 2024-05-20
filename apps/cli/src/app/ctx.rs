//! Runtime context.

use std::fmt::Display;
use std::time::{Duration, Instant};

use remus::Clock;
use rugby::core::dmg::{self, ppu};

/// Runtime context.
#[derive(Debug)]
pub struct Context {
    /// Synchronizer.
    pub clock: Clock,
    /// Cycle counter.
    pub count: Counter,
    /// Elapsed timer.
    pub timer: Instant,
}

/// Counter for emulated cycles.
#[derive(Clone, Debug, Default)]
pub struct Counter {
    /// Total flushed cycles.
    pub cycle: u64,
    /// Intermediate counter.
    pub delta: u32,
}

impl Counter {
    /// Constructs a new `Counter`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the absolute cycle number.
    pub fn cycle(&self) -> u32 {
        #[allow(clippy::cast_possible_truncation)]
        (self.cycle as u32).wrapping_add(self.delta)
    }

    /// Gets the relative cycle delta.
    #[allow(unused)]
    pub fn delta(&self) -> u32 {
        self.delta
    }

    /// Increments the counter by a tick.
    pub fn tick(&mut self) {
        self.delta = self.delta.saturating_add(1);
    }

    /// Flushes intermediate cycles into the absolute cycle counter.
    pub fn flush(&mut self) {
        self.cycle = self.cycle.wrapping_add(self.delta.into());
        self.delta = 0;
    }

    /// Produces clock statistics for the elapsed time.
    pub fn stats(&self, time: Duration) -> Stats {
        let freq = f64::from(self.delta) / time.as_secs_f64();
        Stats {
            freq,
            pace: freq / f64::from(dmg::FREQ),
            rate: freq / f64::from(ppu::RATE),
        }
    }
}

/// Clock statistics.
///
/// Contains statistics on emulation speed normalized to 1 second.
#[derive(Clone, Debug)]
pub struct Stats {
    /// Emulated clock frequency.
    freq: f64,
    /// Full-speed relative pace.
    pace: f64,
    /// Video refresh frame rate.
    rate: f64,
}

#[allow(unused)]
impl Stats {
    /// Gets the emulated clock frequency.
    pub fn freq(&self) -> f64 {
        self.freq
    }

    /// Gets the full-speed relative pace.
    pub fn pace(&self) -> f64 {
        self.pace
    }

    /// Gets the video refresh frame rate.
    pub fn rate(&self) -> f64 {
        self.rate
    }
}

impl Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "frequency: {freq:>7.4} MHz, speedup: {pace:>5.1}%, frames: {rate:>6.2} FPS",
            freq = self.freq / 1e6,
            pace = self.pace * 1e2,
            rate = self.rate
        )
    }
}
