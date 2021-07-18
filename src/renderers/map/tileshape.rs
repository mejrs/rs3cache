/// An iterator that given a shape and tilesize, yields which pixels should be coloured.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct OverlayShape {
    inner: Box<dyn Iterator<Item = (u32, u32)>>,
}

impl OverlayShape {
    /// Constructor for [`OverlayShape`].
    pub fn new(shape: u8, size: u32) -> Self {
        debug_assert!(size.is_power_of_two(), "{} is an invalid size, only 2^n values are allowed.", size);

        let points: Box<dyn Iterator<Item = (u32, u32)>> = match shape {
            0 => Box::new((0..size).flat_map(move |x| (0..size).map(move |y| (x, y)))),

            4 | 39 | 41 => Box::new((0..size).flat_map(move |x| (x..size).map(move |y| (x, y)))),
            5 | 36 | 42 => Box::new((0..size).flat_map(move |x| (0..(size - x)).map(move |y| (x, y)))),
            6 | 37 | 43 => Box::new((0..size).flat_map(move |x| (0..x).map(move |y| (x, y)))),
            7 | 38 | 40 => Box::new((0..size).flat_map(move |x| ((size - x)..size).map(move |y| (x, y)))),

            8 => Box::new((0..(size / 2)).flat_map(move |x| (0..(size - 2 * x)).map(move |y| (x, y)))),
            9 => Box::new((0..size).flat_map(move |x| (0..(x / 2)).map(move |y| (x, y)))),
            10 => Box::new(((size / 2)..size).flat_map(move |x| ((size - 2 * (x - size / 2))..size).map(move |y| (x, y)))),
            11 => Box::new((0..size).flat_map(move |x| (((x + size) / 2)..size).map(move |y| (x, y)))),
            12 => Box::new(((size / 2)..size).flat_map(move |x| (0..(2 * x - size)).map(move |y| (x, y)))),
            13 => Box::new((0..size).flat_map(move |x| ((size - 1 - (x / 2))..size).map(move |y| (x, y)))),
            14 => Box::new((0..(size / 2)).flat_map(move |x| ((2 * x)..size).map(move |y| (x, y)))),
            15 => Box::new((0..size).flat_map(move |x| (0..((size - x) / 2)).map(move |y| (x, y)))),

            16 => Box::new((0..size).flat_map(move |x| ((size.saturating_sub(2 * x))..size).map(move |y| (x, y)))),
            17 => Box::new((0..size).flat_map(move |x| ((x / 2)..size).map(move |y| (x, y)))),
            18 => Box::new((0..size).flat_map(move |x| (0..(2 * (size - x)).clamp(0, size)).map(move |y| (x, y)))),
            19 => Box::new((0..size).flat_map(move |x| (0..((x + size) / 2)).map(move |y| (x, y)))),
            20 => Box::new((0..size).flat_map(move |x| ((2 * x).saturating_sub(size)..size).map(move |y| (x, y)))),
            21 => Box::new((0..size).flat_map(move |x| (0..(size - 1 - (x / 2))).map(move |y| (x, y)))),
            22 => Box::new((0..size).flat_map(move |x| (0..(2 * x).clamp(0, size)).map(move |y| (x, y)))),
            23 => Box::new((0..size).flat_map(move |x| (((size - x) / 2)..size).map(move |y| (x, y)))),

            24 => Box::new((0..size / 2).flat_map(move |x| (0..size).map(move |y| (x, y)))),
            25 => Box::new((0..size).flat_map(move |x| (0..size / 2).map(move |y| (x, y)))),
            26 => Box::new((size / 2..size).flat_map(move |x| (0..size).map(move |y| (x, y)))),
            27 => Box::new((0..size).flat_map(move |x| (size / 2..size).map(move |y| (x, y)))),

            28 => Box::new((0..size / 2).flat_map(move |x| ((size / 2 + x)..size).map(move |y| (x, y)))),
            29 => Box::new((0..size / 2).flat_map(move |x| (0..(size / 2 - x)).map(move |y| (x, y)))),
            30 => Box::new((size / 2..size).flat_map(move |x| (0..(x - size / 2)).map(move |y| (x, y)))),
            31 => Box::new((size / 2..size).flat_map(move |x| ((size + size / 2 - x)..size).map(move |y| (x, y)))),
            32 | 45 => Box::new((0..size).flat_map(move |x| (0..(size / 2 + x).clamp(0, size)).map(move |y| (x, y)))),
            33 | 46 => Box::new((0..size).flat_map(move |x| ((size / 2).saturating_sub(x)..size).map(move |y| (x, y)))),
            34 | 47 => Box::new((0..size).flat_map(move |x| (x.saturating_sub(size / 2)..size).map(move |y| (x, y)))),
            35 | 44 => Box::new((0..size).flat_map(move |x| (0..(size + size / 2 - x).clamp(0, size)).map(move |y| (x, y)))),
            other => unimplemented!("Shape {} is not implemented.", other),
        };

        Self { inner: points }
    }
}

impl Iterator for OverlayShape {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// An iterator that given a shape and tilesize, yields which pixels should not be coloured.
///
/// Complements [`OverlayShape`].
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct UnderlayShape {
    inner: Box<dyn Iterator<Item = (u32, u32)>>,
}

impl UnderlayShape {
    /// Constructor for UnderlayShape.
    pub fn new(shape: Option<u8>, size: u32) -> Self {
        debug_assert!(size.is_power_of_two(), "{} is an invalid size, only 2^n values are allowed.", size);

        let points: Box<dyn Iterator<Item = (u32, u32)>> = match shape {
            None => Box::new((0..size).flat_map(move |x| (0..size).map(move |y| (x, y)))),
            Some(0) => Box::new(std::iter::empty()),

            Some(4) | Some(39) | Some(41) => Box::new((0..size).flat_map(move |x| (0..x).map(move |y| (x, y)))),
            Some(5) | Some(36) | Some(42) => Box::new((0..size).flat_map(move |x| ((size - x)..size).map(move |y| (x, y)))),
            Some(6) | Some(37) | Some(43) => Box::new((0..size).flat_map(move |x| (x..size).map(move |y| (x, y)))),
            Some(7) | Some(38) | Some(40) => Box::new((0..size).flat_map(move |x| (0..(size - x)).map(move |y| (x, y)))),

            Some(8) => Box::new((0..size).flat_map(move |x| ((size.saturating_sub(2 * x))..size).map(move |y| (x, y)))),
            Some(9) => Box::new((0..size).flat_map(move |x| ((x / 2)..size).map(move |y| (x, y)))),
            Some(10) => Box::new((0..size).flat_map(move |x| (0..(2 * (size - x)).clamp(0, size)).map(move |y| (x, y)))),
            Some(11) => Box::new((0..size).flat_map(move |x| (0..((x + size) / 2)).map(move |y| (x, y)))),
            Some(12) => Box::new((0..size).flat_map(move |x| ((2 * x).saturating_sub(size)..size).map(move |y| (x, y)))),
            Some(13) => Box::new((0..size).flat_map(move |x| (0..(size - 1 - (x / 2))).map(move |y| (x, y)))),
            Some(14) => Box::new((0..size).flat_map(move |x| (0..(2 * x).clamp(0, size)).map(move |y| (x, y)))),
            Some(15) => Box::new((0..size).flat_map(move |x| (((size - x) / 2)..size).map(move |y| (x, y)))),

            Some(16) => Box::new((0..(size / 2)).flat_map(move |x| (0..(size - 2 * x)).map(move |y| (x, y)))),
            Some(17) => Box::new((0..size).flat_map(move |x| (0..(x / 2)).map(move |y| (x, y)))),
            Some(18) => Box::new(((size / 2)..size).flat_map(move |x| ((size - 2 * (x - size / 2))..size).map(move |y| (x, y)))),
            Some(19) => Box::new((0..size).flat_map(move |x| (((x + size) / 2)..size).map(move |y| (x, y)))),
            Some(20) => Box::new(((size / 2)..size).flat_map(move |x| (0..(2 * x - size)).map(move |y| (x, y)))),
            Some(21) => Box::new((0..size).flat_map(move |x| ((size - 1 - (x / 2))..size).map(move |y| (x, y)))),
            Some(22) => Box::new((0..(size / 2)).flat_map(move |x| ((2 * x)..size).map(move |y| (x, y)))),
            Some(23) => Box::new((0..size).flat_map(move |x| (0..((size - x) / 2)).map(move |y| (x, y)))),

            Some(24) => Box::new((size / 2..size).flat_map(move |x| (0..size).map(move |y| (x, y)))),
            Some(25) => Box::new((0..size).flat_map(move |x| (size / 2..size).map(move |y| (x, y)))),
            Some(26) => Box::new((0..size / 2).flat_map(move |x| (0..size).map(move |y| (x, y)))),
            Some(27) => Box::new((0..size).flat_map(move |x| (0..size / 2).map(move |y| (x, y)))),

            Some(28) => Box::new((0..size).flat_map(move |x| (0..(size / 2 + x).clamp(0, size)).map(move |y| (x, y)))),
            Some(29) => Box::new((0..size).flat_map(move |x| ((size / 2).saturating_sub(x)..size).map(move |y| (x, y)))),
            Some(30) => Box::new((0..size).flat_map(move |x| (x.saturating_sub(size / 2)..size).map(move |y| (x, y)))),
            Some(31) => Box::new((0..size).flat_map(move |x| (0..(size + size / 2 - x).clamp(0, size)).map(move |y| (x, y)))),
            Some(32) | Some(45) => Box::new((0..size / 2).flat_map(move |x| ((size / 2 + x)..size).map(move |y| (x, y)))),
            Some(33) | Some(46) => Box::new((0..size / 2).flat_map(move |x| (0..(size / 2 - x)).map(move |y| (x, y)))),
            Some(34) | Some(47) => Box::new((size / 2..size).flat_map(move |x| (0..(x - size / 2)).map(move |y| (x, y)))),
            Some(35) | Some(44) => Box::new((size / 2..size).flat_map(move |x| ((size + size / 2 - x)..size).map(move |y| (x, y)))),
            Some(other) => unimplemented!("Shape {} is not implemented.", other),
        };

        Self { inner: points }
    }
}

impl Iterator for UnderlayShape {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod shape_tests {
    use std::collections::HashSet;

    use super::*;

    // Unsafe code in renderers/map.rs depends on this invariant
    #[test]
    fn test_overlay_bounds() {
        let shapes = [
            0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 31, 32, 33, 34, 35, 36, 37, 38, 39,
            40, 41, 42, 43, 44, 45, 46, 47,
        ];
        let sizes = [2, 4, 8, 16, 32, 64];
        for size in &sizes {
            for shape in &shapes {
                for (x, y) in OverlayShape::new(*shape, *size) {
                    if !(x < *size && y < *size) {
                        panic!("{} {} {} {}", x, y, shape, size)
                    }
                }
            }
        }
    }

    #[test]
    #[should_panic]
    fn overlay_unpower_of_two() {
        let _ = OverlayShape::new(0, 13);
    }

    // Unsafe code in renderers/map.rs depends on this invariant
    #[test]
    fn test_underlay_bounds() {
        let shapes = [
            0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 31, 32, 33, 34, 35, 36, 37, 38, 39,
            40, 41, 42, 43, 44, 45, 46, 47,
        ];
        let sizes = [2, 4, 8, 16, 32, 64];
        for size in &sizes {
            for (x, y) in UnderlayShape::new(None, *size) {
                if !(x < *size && y < *size) {
                    panic!("{} {} {}", x, y, size)
                }
            }
            for shape in &shapes {
                for (x, y) in UnderlayShape::new(Some(*shape), *size) {
                    if !(x < *size && y < *size) {
                        panic!("{} {} {} {}", x, y, shape, size)
                    }
                }
            }
        }
    }

    #[test]
    #[should_panic]
    fn underlay_unpower_of_two() {
        let _ = UnderlayShape::new(Some(0), 13);
    }

    #[test]
    fn are_complement() {
        let shapes = [
            0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 31, 32, 33, 34, 35, 36, 37, 38, 39,
            40, 41, 42, 43, 44, 45, 46, 47,
        ];
        let sizes = [2, 4, 8, 16, 32, 64];
        for size in &sizes {
            for shape in &shapes {
                let mut o = OverlayShape::new(*shape, *size).into_iter().collect::<Vec<_>>();
                let mut u = UnderlayShape::new(Some(*shape), *size).into_iter().collect::<Vec<_>>();
                let total_length = o.len() + u.len();
                assert_eq!(total_length as u32, size * size);

                let mut uniq = HashSet::new();
                o.append(&mut u);
                assert!(o.into_iter().all(move |x| uniq.insert(x)));
            }
        }
    }
}
