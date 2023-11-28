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
        let expect = [126, 127, -128, -127];
        assert_eq!(found, expect);
    }

    #[test]
    fn from_range_from_works() {
        let range = Derange::from(..4);
        let found = range.into_iter().collect::<Vec<u8>>();
        let expect = [0, 1, 2, 3];
        assert_eq!(found, expect);
    }

    #[test]
    fn from_range_full_works() {
        let range = Derange::from(..);
        let found = range.into_iter().collect::<Vec<i8>>();
        let expect = [
            -128, -127, -126, -125, -124, -123, -122, -121, -120, -119, -118, -117, -116, -115,
            -114, -113, -112, -111, -110, -109, -108, -107, -106, -105, -104, -103, -102, -101,
            -100, -99, -98, -97, -96, -95, -94, -93, -92, -91, -90, -89, -88, -87, -86, -85, -84,
            -83, -82, -81, -80, -79, -78, -77, -76, -75, -74, -73, -72, -71, -70, -69, -68, -67,
            -66, -65, -64, -63, -62, -61, -60, -59, -58, -57, -56, -55, -54, -53, -52, -51, -50,
            -49, -48, -47, -46, -45, -44, -43, -42, -41, -40, -39, -38, -37, -36, -35, -34, -33,
            -32, -31, -30, -29, -28, -27, -26, -25, -24, -23, -22, -21, -20, -19, -18, -17, -16,
            -15, -14, -13, -12, -11, -10, -9, -8, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6,
            7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
            29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
            51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72,
            73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94,
            95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
            113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127,
        ];
        assert_eq!(found, expect);
    }

    #[test]
    fn from_range_inclusive_works() {
        let range = Derange::from(254..=2);
        let found = range.into_iter().collect::<Vec<u8>>();
        let expect = [254, 255, 0, 1, 2];
        assert_eq!(found, expect);
    }

    #[test]
    fn from_range_to_works() {
        let range = Derange::from(..4);
        let found = range.into_iter().collect::<Vec<u8>>();
        let expect = [0, 1, 2, 3];
        assert_eq!(found, expect);
    }

    #[test]
    fn from_range_to_inclusive_works() {
        let range = Derange::from(..=4);
        let found = range.into_iter().collect::<Vec<u8>>();
        let expect = [0, 1, 2, 3, 4];
        assert_eq!(found, expect);
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
