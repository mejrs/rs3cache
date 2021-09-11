/// An iterator to maintain state while iterating another iterator.
///
/// `Accumulate` is created by the [`accumulate`] method on [`Iterator`]. See its
/// documentation for more.
///
/// [`accumulate`]: trait.Accumulator.html#method.accumulate
/// [`Iterator`]: ../../std/iter/trait.Iterator.html
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Accumulate<I, F>
where
    I: Iterator,
    F: Fn(I::Item, I::Item) -> I::Item,
{
    accum: Option<I::Item>,
    underlying: I,
    acc_fn: F,
}

impl<I, F> Iterator for Accumulate<I, F>
where
    I: Iterator,
    I::Item: Copy,
    F: Fn(I::Item, I::Item) -> I::Item,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.underlying.next() {
            Some(x) => {
                let new_accum = match self.accum {
                    // Apply function to item-so-far and current item
                    Some(accum) => (self.acc_fn)(accum, x),

                    // This is the first item
                    None => x,
                };
                self.accum = Some(new_accum);
                Some(new_accum)
            }
            None => None,
        }
    }
}

/// An iterator to maintain state while iterating another iterator.
///
/// `Accumulate` is created by the [`accumulate`] method on [`Iterator`]. See its
/// documentation for more.
///
/// [`accumulate`]: trait.Accumulator.html#method.accumulate
/// [`Iterator`]: ../../std/iter/trait.Iterator.html
pub trait Accumulator: Iterator {
    /// An iterator adaptor that yields the computation of the closure `F` over the
    /// accumulated value and the next element of the preceding iterator.
    ///
    /// `accumulate()` takes a function or closure with two arguments,
    /// the first being the previous yielded value and the second an element from the preceding iterator.
    /// The iterator maintains the state of the last yielded value.
    ///
    /// The first element of the preceding iterator is yielded as-is.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use rs3cache::utils::adapters::Accumulator;
    ///
    /// let mut iter = (1..6).accumulate(|x, y| x + y);
    ///
    /// assert_eq!(iter.next(), Some(1));  // 1
    /// assert_eq!(iter.next(), Some(3));  // 1+2
    /// assert_eq!(iter.next(), Some(6));  // (1+2) + 3
    /// assert_eq!(iter.next(), Some(10)); // (1+2+3) + 4
    /// assert_eq!(iter.next(), Some(15)); // (1+2+3+4) + 5
    /// assert_eq!(iter.next(), None);
    /// ```
    fn accumulate<F>(self, f: F) -> Accumulate<Self, F>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item,
        Self::Item: Copy,
        Self: Sized,
        Self: Iterator,
    {
        Accumulate {
            accum: None,
            underlying: self,
            acc_fn: f,
        }
    }
}

impl<I: Iterator> Accumulator for I {}

/// An iterator that returns pair values ofn the preceding iterator.
///
/// `Pairwise` is created by the [`pairwise`] method on [`Iterator`]. See its
/// documentation for more.
///
/// [`pairwise`]: trait.Pairwisor.html#method.pairwise
/// [`Iterator`]: trait.Iterator.html
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Pairwise<I>
where
    I: Iterator,
{
    state: Option<I::Item>,
    underlying_iterator: I,
}

impl<I> Iterator for Pairwise<I>
where
    I: Iterator,
    I::Item: Copy,
{
    type Item = (I::Item, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match self.underlying_iterator.next() {
            Some(x) => match self.state {
                Some(y) => {
                    self.state = Some(x);
                    Some((y, x))
                }
                None => match self.underlying_iterator.next() {
                    Some(y) => {
                        self.state = Some(y);
                        Some((x, y))
                    }
                    None => None,
                },
            },
            None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.underlying_iterator.size_hint()
    }
}

/// An iterator that returns pair values of the preceding iterator.
///
/// `Pairwise` is created by the [`pairwise`] method on [`Iterator`]. See its
/// documentation for more.
///
/// [`pairwise`]: trait.Pairwisor.html#method.pairwise
/// [`Iterator`]: trait.Iterator.html
pub trait Pairwisor: Iterator {
    /// An iterator adaptor that yields the preceding iterator in pairs.
    ///
    /// Similar to the [`windows`] method on [`slices`], but with owned elements.
    ///
    /// The first element of the preceding iterator is yielded unchanged.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use rs3cache::utils::adapters::Pairwisor;
    ///
    /// let mut iter = (0..6).pairwise();
    ///
    /// assert_eq!(iter.next(), Some((0, 1)));
    /// assert_eq!(iter.next(), Some((1, 2)));
    /// assert_eq!(iter.next(), Some((2, 3)));
    /// assert_eq!(iter.next(), Some((3, 4)));
    /// assert_eq!(iter.next(), Some((4, 5)));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// [`windows`]: ../../std/primitive.slice.html#method.windows
    /// [`slices`]: ../../std/primitive.slice.html
    fn pairwise(self) -> Pairwise<Self>
    where
        Self::Item: Copy,
        Self: Sized,
        Self: Iterator,
    {
        Pairwise {
            state: None,
            underlying_iterator: self,
        }
    }
}

impl<I: Iterator> Pairwisor for I {}

#[cfg(test)]
mod accumulator_tests {
    use super::Accumulator;

    #[test]
    fn test_accumulator_cumulative() {
        let result: Vec<i32> = (1..6).accumulate(|x, y| x + y).collect();
        let expected_result: Vec<i32> = vec![1, 3, 6, 10, 15];
        assert!(result == expected_result)
    }

    #[test]
    fn test_accumulator_rolling_factorial() {
        let result: Vec<i32> = (1..6).accumulate(|x, y| x * y).collect();
        let expected_result: Vec<i32> = vec![1, 2, 6, 24, 120];
        assert!(result == expected_result)
    }
}

#[cfg(test)]
mod pairwise_tests {
    use super::Pairwisor;

    #[test]
    fn test_pairwise() {
        let result: Vec<(i32, i32)> = (0..6).pairwise().collect();
        let expected_result: Vec<(i32, i32)> = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
        assert_eq!(result, expected_result)
    }

    #[test]
    fn test_empty_pairwise() {
        let result: Vec<(i32, i32)> = Vec::new().into_iter().pairwise().collect();
        let expected_result: Vec<(i32, i32)> = Vec::new();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn test_single_pairwise() {
        let result: Vec<(i32, i32)> = (0..1).pairwise().collect();
        let expected_result: Vec<(i32, i32)> = Vec::new();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn test_double_pairwise() {
        let result: Vec<(i32, i32)> = (0..2).pairwise().collect();
        let expected_result: Vec<(i32, i32)> = vec![(0, 1)];
        assert_eq!(result, expected_result)
    }
}
