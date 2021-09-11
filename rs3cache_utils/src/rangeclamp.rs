use std::{
    fmt::Debug,
    ops::{Add, Bound, Range, RangeBounds},
};

/// Provides the `clamp()` method for [`Range`], clamping it to a given interval.
pub trait RangeClamp {
    /// Clamps a range to a given `start_bound` and `end_bound`.
    fn clamp(self, start_bound: Self::Item, end_bound: Self::Item) -> Range<Self::Item>
    where
        Self::Item: Debug + Ord + Copy + Add,
        Self: Sized + Iterator + RangeBounds<<Self as Iterator>::Item>,
    {
        let start = match self.start_bound() {
            Bound::Unbounded => start_bound,
            Bound::Included(start) => *start.clamp(&start_bound, &end_bound),
            Bound::Excluded(_) => unreachable!(),
        };

        let end = match self.end_bound() {
            Bound::Unbounded => end_bound,
            Bound::Included(_) => unimplemented!(),
            Bound::Excluded(end) => *end.clamp(&start_bound, &end_bound),
        };

        Range { start, end }
    }
}

impl<I: Iterator> RangeClamp for I {}
