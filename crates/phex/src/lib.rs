//! Pretty hexadecimal printing for slices of bytes.
//!
//! Wrap any blob (big lump of bytes) in a [`Printer`] object to enable a simple
//! and readable [`Display`] implementation.

#![warn(clippy::pedantic)]

use std::fmt::{Display, Write};
use std::marker::PhantomData;

use num::Unsigned;

/// Formatter for slices of bytes that implements [`Display`].
#[derive(Debug)]
pub struct Printer<'a, W: Unsigned>(usize, &'a [u8], PhantomData<W>);

impl<'a, W: Unsigned> Printer<'a, W> {
    /// Constructs a new `Printer` for the provided data.
    #[must_use]
    pub fn new(offset: usize, data: &'a [u8]) -> Self {
        Self(offset, data, PhantomData)
    }
}

macro_rules! add_impl {
    ($(($t:ty, $i:expr))*) => ($(
        impl<'a> Printer<'a, $t> {
            fn display(&self) -> impl Display {
                let &Self(offset, data, _) = self;
                Internal {
                    offset,
                    data,
                    wordsz: std::mem::size_of::<$t>(),
                    linesz: $i,
                }.display()
            }
        }

        impl<'a> Display for Printer<'a, $t> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.display())
            }
        }
    )*)
}

add_impl! { (usize, 2) (u8, 8) (u16, 4) (u32, 4) (u64, 2) (u128, 1) }

#[derive(Debug)]
struct Internal<'a> {
    offset: usize,
    data: &'a [u8],
    wordsz: usize,
    linesz: usize,
}

impl<'a> Internal<'a> {
    #[inline]
    fn display(self) -> impl Display {
        // Destructure self into constituent parts
        let Self {
            offset,
            data,
            wordsz,
            linesz,
        } = self;
        let end = offset + data.len();

        // Calculate maximum width for formatted addresses
        let width = format!("{end:#x}").len();

        // Prepare format string
        let mut f = String::new();
        // Set up repeat tracking
        let mut last = None;
        let mut repeat = None;
        let mut skip = false;

        let mut data = data.chunks(linesz * wordsz).enumerate().peekable();
        while let Some((idx, line)) = data.next() {
            // Check if this line repeats a single padding byte
            let pad = match line {
                [head, tail @ ..] => tail
                    .iter()
                    .all(|byte| byte == head)
                    .then_some(head)
                    .copied(),
                _ => None,
            };
            // Ignore repeated lines
            if skip && pad == repeat {
                continue;
            }
            // Escalate if this line repeats previous line's last byte
            if last == pad {
                repeat = pad;
            }
            // Check if we should draw this line with ellipses
            let mut ellipses = !skip && pad.is_some() && repeat == pad;
            // De-escalate if the next line breaks the pattern
            if ellipses && Some(line) != data.peek().map(|(_, line)| line).copied() {
                ellipses = false;
            }

            // Calculate this line's starting address
            let addr = offset + (idx * linesz * wordsz);

            // Write newline between lines
            if idx != 0 {
                writeln!(f).unwrap();
            }
            // Write address prefix
            if ellipses {
                // Started repetition, write ellipses instead
                write!(f, "{}:", ".".repeat(width)).unwrap();
                // After printing this line, skip all future repeated lines
                skip = true;
            } else {
                // Format address as usual
                write!(f, "{addr:#0width$x}:",).unwrap();
                // Since this line is being printed usually, we must update the
                // repeat byte in order to check if the next line is a repeat.
                repeat = pad;
                // However, the next line is guaranteed to be printed, since
                // even if it repeats we must first print ellipses.
                skip = false;
            }
            // Write line contents
            for word in line.chunks(wordsz) {
                write!(f, " ").unwrap();
                for &byte in word {
                    write!(f, "{byte:02x}").unwrap();
                }
            }

            // Store this line's last byte
            last = line.iter().last().copied();
        }

        f
    }
}
