use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Shared clock running state.
type Runnable = Arc<Mutex<State>>;

/// Internal state of a [`Clock`].
#[derive(Debug, Default)]
enum State {
    /// Causes the clock to run.
    #[default]
    Play,
    /// Prevents the clock from ticking.
    Stop,
    /// Drops the run-thread.
    ///
    /// # Note
    ///
    /// This renders the clock invalid, after which calling [`Iterator::next`]
    /// may result in a panic.
    Drop,
}

/// Clock signal generator.
///
/// An [`Iterator`] that ensures values are yielded on average[^1] according to
/// the [elapsed real time]. `Clock` internally handles the ilogic of keeping
/// track of the "ticks" (rising edges) of the clock signal.
///
/// [^1]: As `Clock` internally uses the host machine's [`sleep`](thread::sleep)
///       functionality, the host OS may elect to sleep for longer than the
///       specified duration. To handle this, upon waking from sleep the `Clock`
///       will check how cycles it has slept for and tick accordingly to make up
///       missed cycles.
///
/// [elapsed real time]: https://en.wikipedia.org/wiki/Elapsed_real_time
#[derive(Debug)]
pub struct Clock {
    tick: Duration,
    play: Runnable,
    recv: Receiver<()>,
    join: JoinHandle<()>,
}

impl Clock {
    /// Constructs a `Clock` that ticks at the provided frequency.
    #[must_use]
    pub fn with_freq(freq: u32) -> Self {
        // Calculate this frequency's corresponding duration.
        let dx = Self::to_period(freq);
        // Start the run-thread
        Self::play(dx)
    }

    /// Constructs a `Clock` that ticks at the provided duration.
    #[must_use]
    pub fn with_period(tick: Duration) -> Self {
        // Start the run-thread
        Self::play(tick)
    }

    /// Spins up a run-thread for execution.
    fn play(tick: Duration) -> Self {
        // Create a channel to forward clock ticks
        let (send, recv) = mpsc::channel();
        // Share an enable signal
        let play = Runnable::default();
        // Spin up the runner thread
        let join = {
            let child = Child {
                tick,
                play: play.clone(),
                send,
            };
            thread::spawn(move || child.run())
        };
        // Return the constructed clock
        Clock {
            tick,
            play,
            recv,
            join,
        }
    }

    /// Gets this `Clock`'s period.
    #[must_use]
    pub fn period(&self) -> Duration {
        self.tick
    }

    /// Gets this `Clock`'s frequency.
    #[must_use]
    pub fn freq(&self) -> u32 {
        Self::to_freq(self.tick)
    }

    /// Converts a frequency into a period.
    fn to_period(freq: u32) -> Duration {
        Duration::from_secs_f64(f64::from(freq).recip())
    }

    /// Converts a period into a frequency.
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn to_freq(period: Duration) -> u32 {
        period.as_secs_f64().recip().round() as u32
    }

    /// Pauses the clock, preventing iterations from progressing.
    ///
    /// # Note
    ///
    /// Does nothing if the clock is already paused. Upon being paused, cycles
    /// already clocked-in by the run-thread will still run.
    pub fn pause(&mut self) {
        if let Ok(mut play) = self.play.try_lock() {
            *play = State::Stop;
        }
    }

    /// Resumes the clock, iterating at the previously set frequency.
    ///
    /// # Note
    ///
    /// Does nothing if the clock is already running.
    pub fn resume(&mut self) {
        if let Ok(mut play) = self.play.try_lock() {
            *play = State::Play;
        }
        self.join.thread().unpark();
    }
}

impl Drop for Clock {
    fn drop(&mut self) {}
}

impl Iterator for Clock {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        self.recv.recv().ok()
    }
}

/// Runner thread data.
#[derive(Debug)]
struct Child {
    tick: Duration,
    play: Runnable,
    send: Sender<()>,
}

impl Child {
    /// Main function of the runner thread.
    ///
    /// This will continually sends ticks at the provided frequency to the parent
    /// thread, unless it is stopped, or the clock is dropped (at which point it
    /// exits).
    fn run(&self) {
        // Deconstruct child
        let Self { tick, play, send } = self;
        // Keep track of fractional missed cycles
        let mut rem = 0;

        loop {
            // Loop until paused externally
            let mut play = play.lock().unwrap();
            match *play {
                // Send ticks at a regular frequency
                State::Play => {
                    // Check the time before going to sleep
                    // NOTE: Due to OS scheduling, the call to `thread::sleep()` may
                    //       last longer than the specified duration. Because of
                    //       this, we must record how many cycles were missed.
                    let now = Instant::now();
                    // Sleep for the specified duration
                    thread::sleep(*tick);
                    // Calculate how many cycles were slept through
                    let cycles = {
                        // Get elapsed (with remainder), duration in nanoseconds
                        let now = now.elapsed().as_nanos() + rem;
                        let per = tick.as_nanos();
                        // Calculate elapsed cycle remainder
                        rem = now % per;
                        // Calculate elapsed complete cycles
                        now / per
                    };
                    // Clock in elapsed cycles. Run until failure (usually caused by
                    // the receiver hanging up).
                    if (0..cycles).any(|_| send.send(()).is_err()) {
                        // Error encountered, pause the clock
                        *play = State::Drop;
                        break;
                    }
                }
                // Park this thread, as it cannot progress while paused
                State::Stop => thread::park(),
                // Return, since the clock has been dropped
                State::Drop => return,
            }
        }
    }
}
