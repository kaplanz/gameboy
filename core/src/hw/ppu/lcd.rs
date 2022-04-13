use std::fmt::Display;
use std::ops::{Deref, DerefMut};

use super::pixel::Colour;
use crate::SCREEN;

#[derive(Debug)]
pub struct Screen([Colour; SCREEN.len()]);

impl Default for Screen {
    fn default() -> Self {
        Self([Default::default(); SCREEN.len()])
    }
}

impl Deref for Screen {
    type Target = [Colour];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌{}┐", "─".repeat(SCREEN.width))?;
        let rows = self.chunks_exact(SCREEN.width);
        let even = rows.clone().step_by(2);
        let odd = rows.clone().step_by(2).skip(1);
        let rows = even.zip(odd);
        for (even, odd) in rows {
            writeln!(
                f,
                "│{}│",
                even.iter()
                    .zip(odd)
                    .map(|(p0, p1)| match (p0, p1) {
                        (Colour::C0, Colour::C0) => ' ',
                        (Colour::C0, _) => '▄',
                        (_, Colour::C0) => '▀',
                        (_, _) => '█',
                    })
                    .collect::<String>()
            )?;
        }
        write!(f, "└{}┘", "─".repeat(SCREEN.width))
    }
}
