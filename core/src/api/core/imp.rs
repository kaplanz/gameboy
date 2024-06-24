use super::{has, Core, Inside, InsideMut};

impl<'a, C: Core + has::Audio> Inside<'a, C> {
    /// Borrow the core's audio.
    #[must_use]
    pub fn audio(self) -> &'a C::Audio {
        self.0.audio()
    }
}

impl<'a, C: Core + has::Audio> InsideMut<'a, C> {
    /// Mutably borrow the core's audio.
    #[must_use]
    pub fn audio(self) -> &'a mut C::Audio {
        self.0.audio_mut()
    }
}

impl<'a, C: Core + has::Joypad> Inside<'a, C> {
    /// Borrow the core's joypad.
    #[must_use]
    pub fn joypad(self) -> &'a C::Joypad {
        self.0.joypad()
    }
}

impl<'a, C: Core + has::Joypad> InsideMut<'a, C> {
    /// Mutably borrow the core's joypad.
    #[must_use]
    pub fn joypad(self) -> &'a mut C::Joypad {
        self.0.joypad_mut()
    }
}

impl<'a, C: Core + has::Processor> Inside<'a, C> {
    /// Borrow the core's processor.
    #[must_use]
    pub fn proc(self) -> &'a C::Proc {
        self.0.proc()
    }
}

impl<'a, C: Core + has::Processor> InsideMut<'a, C> {
    /// Mutably borrow the core's processor.
    #[must_use]
    pub fn proc(self) -> &'a mut C::Proc {
        self.0.proc_mut()
    }
}

impl<'a, C: Core + has::Serial> Inside<'a, C> {
    /// Borrow the core's serial.
    #[must_use]
    pub fn serial(self) -> &'a C::Serial {
        self.0.serial()
    }
}

impl<'a, C: Core + has::Serial> InsideMut<'a, C> {
    /// Mutably borrow the core's serial.
    #[must_use]
    pub fn serial(self) -> &'a mut C::Serial {
        self.0.serial_mut()
    }
}

impl<'a, C: Core + has::Video> Inside<'a, C> {
    /// Borrow the core's video.
    #[must_use]
    pub fn video(self) -> &'a C::Video {
        self.0.video()
    }
}

impl<'a, C: Core + has::Video> InsideMut<'a, C> {
    /// Mutably borrow the core's video.
    #[must_use]
    pub fn video(self) -> &'a mut C::Video {
        self.0.video_mut()
    }
}
