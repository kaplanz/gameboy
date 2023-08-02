use std::fmt::{Display, Write};
use std::marker::PhantomData;

use num::Unsigned;

#[derive(Debug)]
pub struct Printer<'a, W: Unsigned>(usize, &'a [u8], PhantomData<W>);

impl<'a, W: Unsigned> Printer<'a, W> {
    pub fn new(start: usize, data: &'a [u8]) -> Self {
        Self(start, data, PhantomData)
    }
}

macro_rules! add_impl {
    ($(($t:ty, $i:expr))*) => ($(
        impl<'a> Printer<'a, $t> {
            #[allow(unused)]
            pub fn display(self) -> impl Display {
                let Self(start, data, _) = self;
                Internal {
                    start,
                    data,
                    wordsz: std::mem::size_of::<$t>(),
                    linesz: $i,
                }.display()
            }
        }
    )*)
}

add_impl! { (usize, 2) (u8, 8) (u16, 4) (u32, 4) (u64, 2) (u128, 1) }

#[derive(Debug)]
struct Internal<'a> {
    start: usize,
    data: &'a [u8],
    wordsz: usize,
    linesz: usize,
}

impl<'a> Internal<'a> {
    #[inline]
    fn display(self) -> impl Display {
        // Destructure self into constituent parts
        let Self {
            start,
            data,
            wordsz,
            linesz,
        } = self;
        let end = start + data.len();

        // Calculate maximum width for formatted addresses
        let width = format!("{end:#x}").len();

        // Prepare format string
        let mut f = String::new();
        // Set up repeat tracking
        let mut repeat = None;
        let mut skip = false;

        for (idx, line) in data.chunks(linesz * wordsz).enumerate() {
            // Check if this line repeats a single padding byte
            let pad = match line {
                [head, tail @ ..] => tail.iter().all(|byte| byte == head).then_some(head),
                _ => None,
            };
            // Ignore repeated lines
            if skip && pad == repeat {
                continue;
            }

            // Calculate this line's starting address
            let addr = start + (idx * linesz * wordsz);

            // Write newline between lines
            if idx != 0 {
                writeln!(f).unwrap();
            }
            // Write address prefix
            if !skip && pad.is_some() && repeat == pad {
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
        }

        f
    }
}
