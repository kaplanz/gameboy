//! Wrapping range iteration.
//!
//! # Examples
//!
//! ```
//! use derange::Derange;
//!
//! // Create a wrapping range
//! let range = Derange::<u8>::from(250..5);
//!
//! // Perform iteration
//! for i in range {
//!   // -- snip --
//! }
//! ```

#![warn(clippy::pedantic)]
#![allow(clippy::reversed_empty_ranges)]

use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use num::traits::WrappingSub;
use num::{Bounded, Integer};

/// A wrapping range bounded (inclusively) below and above.
///
/// `Derange` can be used to create iterators from ranges that wrap, or
/// overflow, about the type's maximum and minimum bounds.
#[derive(Clone, Debug)]
pub struct Derange<I>
where
    I: Bounded + Clone + Copy + Integer,
    RangeInclusive<I>: Iterator<Item = I>,
{
    /// The lower bound of the range (inclusive).
    pub start: I,
    /// The upper bound of the range (inclusive).
    pub end: I,
}

impl<I> From<Range<I>> for Derange<I>
where
    I: Bounded + Clone + Copy + Integer + WrappingSub + 'static,
    RangeInclusive<I>: Iterator<Item = I>,
{
    fn from(Range { start, mut end }: Range<I>) -> Self {
        end = end.wrapping_sub(&I::one());
        Self { start, end }
    }
}

impl<I> From<RangeFrom<I>> for Derange<I>
where
    I: Bounded + Clone + Copy + Integer + 'static,
    RangeInclusive<I>: Iterator<Item = I>,
{
    fn from(RangeFrom { start }: RangeFrom<I>) -> Self {
        let end = I::max_value();
        Self { start, end }
    }
}

impl<I> From<RangeFull> for Derange<I>
where
    I: Bounded + Clone + Copy + Integer + 'static,
    RangeInclusive<I>: Iterator<Item = I>,
{
    fn from(RangeFull: RangeFull) -> Self {
        let start = I::min_value();
        let end = I::max_value();
        Self { start, end }
    }
}

impl<I> From<RangeInclusive<I>> for Derange<I>
where
    I: Bounded + Clone + Copy + Integer + 'static,
    RangeInclusive<I>: Iterator<Item = I>,
{
    fn from(range: RangeInclusive<I>) -> Self {
        let (&start, &end) = (range.start(), range.end());
        Self { start, end }
    }
}

impl<I> From<RangeTo<I>> for Derange<I>
where
    I: Bounded + Clone + Copy + Integer + WrappingSub + 'static,
    RangeInclusive<I>: Iterator<Item = I>,
{
    fn from(RangeTo { mut end }: RangeTo<I>) -> Self {
        let start = I::min_value();
        end = end.wrapping_sub(&I::one());
        Self { start, end }
    }
}

impl<I> From<RangeToInclusive<I>> for Derange<I>
where
    I: Bounded + Clone + Copy + Integer + 'static,
    RangeInclusive<I>: Iterator<Item = I>,
{
    fn from(RangeToInclusive { end }: RangeToInclusive<I>) -> Self {
        let start = I::min_value();
        Self { start, end }
    }
}

impl<I> IntoIterator for Derange<I>
where
    I: Bounded + Clone + Copy + Integer + 'static,
    RangeInclusive<I>: Iterator<Item = I>,
{
    type Item = I;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    #[rustfmt::skip]
    fn into_iter(self) -> Self::IntoIter {
        let Self { start, end } = self;
        let min = I::min_value();
        let max = I::max_value();
        if start <= end {
            Box::new(start..=end)
        } else {
            Box::new((start..=max).chain(min..=end))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_range_works() {
        let range = Derange::from(126..-126);
        let found = range.into_iter().collect::<Vec<i8>>();
        let truth = [126, 127, -128, -127];
        assert_eq!(found, truth);
    }

    #[test]
    fn from_range_from_works() {
        let range = Derange::from(..4);
        let found = range.into_iter().collect::<Vec<u8>>();
        let truth = [0, 1, 2, 3];
        assert_eq!(found, truth);
    }

    #[test]
    fn from_range_full_works() {
        let range = Derange::from(..);
        let found = range.into_iter().collect::<Vec<i8>>();
        #[allow(clippy::cast_possible_truncation)]
        let truth: [_; 256] = std::array::from_fn(|i| (i as i8).wrapping_add(-128));
        assert_eq!(found, truth);
    }

    #[test]
    fn from_range_inclusive_works() {
        let range = Derange::from(254..=2);
        let found = range.into_iter().collect::<Vec<u8>>();
        let truth = [254, 255, 0, 1, 2];
        assert_eq!(found, truth);
    }

    #[test]
    fn from_range_to_works() {
        let range = Derange::from(..4);
        let found = range.into_iter().collect::<Vec<u8>>();
        let truth = [0, 1, 2, 3];
        assert_eq!(found, truth);
    }

    #[test]
    fn from_range_to_inclusive_works() {
        let range = Derange::from(..=4);
        let found = range.into_iter().collect::<Vec<u8>>();
        let truth = [0, 1, 2, 3, 4];
        assert_eq!(found, truth);
    }

    #[test]
    fn iter_count_works() {
        assert_eq!(Derange::<u8>::from(0..4).into_iter().count(), 4);
        assert_eq!(Derange::<u8>::from(..).into_iter().count(), 256);
        assert_eq!(Derange::<u8>::from(252..).into_iter().count(), 4);
        assert_eq!(Derange::<u8>::from(0..=4).into_iter().count(), 5);
        assert_eq!(Derange::<u8>::from(..4).into_iter().count(), 4);
        assert_eq!(Derange::<u8>::from(..=4).into_iter().count(), 5);
    }

    #[test]
    fn edge_works() {
        // One item
        assert_eq!(Derange::<u8>::from(0..1).into_iter().count(), 1);
        assert_eq!(Derange::<u8>::from(255..).into_iter().count(), 1);
        assert_eq!(Derange::<u8>::from(0..=0).into_iter().count(), 1);
        assert_eq!(Derange::<u8>::from(..1).into_iter().count(), 1);
        assert_eq!(Derange::<u8>::from(..=0).into_iter().count(), 1);
    }
}
