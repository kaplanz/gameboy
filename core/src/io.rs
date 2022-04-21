//! Player I/O interfaces.

/// Joypad input.
pub mod joypad {
    use std::fmt::Debug;

    /// Input interface.
    pub trait Input: Copy + Clone + Debug {}
}

/// LCD screen.
pub mod screen {
    use std::fmt::Debug;
    use std::ops::{Deref, DerefMut};

    /// Screen data.
    #[derive(Debug)]
    pub struct Screen<P: Pixel, const L: usize>([P; L]);

    impl<P: Pixel, const L: usize> Default for Screen<P, L> {
        fn default() -> Self {
            Self([Default::default(); L])
        }
    }

    impl<P: Pixel, const L: usize> Deref for Screen<P, L> {
        type Target = [P];

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<P: Pixel, const L: usize> DerefMut for Screen<P, L> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<P: Pixel, const L: usize> From<[P; L]> for Screen<P, L> {
        fn from(buf: [P; L]) -> Self {
            Self(buf)
        }
    }

    /// Resolution info.
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

    /// Pixel values.
    pub trait Pixel: Copy + Clone + Debug + Default + Into<usize> {}
}
