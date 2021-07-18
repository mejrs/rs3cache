use std::ops::{Deref, Range};
use std::rc::Rc;

pub struct SharedSlice<T> {
    inner: Rc<[T]>,
    start: usize,
    end: usize,
}

impl<T> SharedSlice<T> {
    pub fn new(slice: Box<[T]>) -> Self {
        let start = 0;
        let end = slice.len();
        Self {
            inner: slice.into(),
            start,
            end,
        }
    }

    pub fn from(&self, range: Range<usize>) -> Self {
        let Range { start, end } = range;

        assert!(
            start >= self.start && end <= self.end,
            "Attempted to share range {}..{}, but the valid range is {}_{}",
            start,
            end,
            self.start,
            self.end
        );

        Self {
            inner: Rc::clone(&self.inner),
            start,
            end,
        }
    }
}

impl<T> Deref for SharedSlice<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.inner[self.start..self.end]
    }
}