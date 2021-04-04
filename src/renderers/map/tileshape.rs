/// An iterator that given a shape and tilesize, yields which pixels should be coloured.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct OverlayShape {
    inner: Vec<(u32, u32)>,
}

impl OverlayShape {
    /// Constructor for [`OverlayShape`].
    pub fn new(shape: u8, size: u32) -> Self {
        debug_assert!(size.is_power_of_two(), "{} is an invalid size Only 2^n values are allowed.", size);

        let points: Vec<(u32, u32)> = match shape {
            0 => (0..size).flat_map(|x| (0..size).map(move |y| (x, y))).collect(),

            4 | 39 | 41 => (0..size).flat_map(|x| (x..size).map(move |y| (x, y))).collect(),
            5 | 36 | 42 => (0..size).flat_map(|x| (0..(size - x)).map(move |y| (x, y))).collect(),
            6 | 37 | 43 => (0..size).flat_map(|x| (0..x).map(move |y| (x, y))).collect(),
            7 | 38 | 40 => (0..size).flat_map(|x| ((size - x)..size).map(move |y| (x, y))).collect(),

            8 => (0..(size / 2)).flat_map(|x| (0..(size - 2 * x)).map(move |y| (x, y))).collect(),
            9 => (0..size).flat_map(|x| (0..(x / 2)).map(move |y| (x, y))).collect(),
            10 => ((size / 2)..size)
                .flat_map(|x| ((size - 2 * (x - size / 2))..size).map(move |y| (x, y)))
                .collect(),
            11 => (0..size).flat_map(|x| (((x + size) / 2)..size).map(move |y| (x, y))).collect(),
            12 => ((size / 2)..size).flat_map(|x| (0..(2 * x - size)).map(move |y| (x, y))).collect(),
            13 => (0..size).flat_map(|x| ((size - 1 - (x / 2))..size).map(move |y| (x, y))).collect(),
            14 => (0..(size / 2)).flat_map(|x| ((2 * x)..size).map(move |y| (x, y))).collect(),
            15 => (0..size).flat_map(|x| (0..((size - x) / 2)).map(move |y| (x, y))).collect(),

            16 => (0..size)
                .flat_map(|x| ((size.saturating_sub(2 * x))..size).map(move |y| (x, y)))
                .collect(),
            17 => (0..size).flat_map(|x| ((x / 2)..size).map(move |y| (x, y))).collect(),
            18 => (0..size)
                .flat_map(|x| (0..(2 * (size - x)).clamp(0, size)).map(move |y| (x, y)))
                .collect(),
            19 => (0..size).flat_map(|x| (0..((x + size) / 2)).map(move |y| (x, y))).collect(),
            20 => (0..size)
                .flat_map(|x| ((2 * x).saturating_sub(size)..size).map(move |y| (x, y)))
                .collect(),
            21 => (0..size).flat_map(|x| (0..(size - 1 - (x / 2))).map(move |y| (x, y))).collect(),
            22 => (0..size).flat_map(|x| (0..(2 * x).clamp(0, size)).map(move |y| (x, y))).collect(),
            23 => (0..size).flat_map(|x| (((size - x) / 2)..size).map(move |y| (x, y))).collect(),

            24 => (0..size / 2).flat_map(|x| (0..size).map(move |y| (x, y))).collect(),
            25 => (0..size).flat_map(|x| (0..size / 2).map(move |y| (x, y))).collect(),
            26 => (size / 2..size).flat_map(|x| (0..size).map(move |y| (x, y))).collect(),
            27 => (0..size).flat_map(|x| (size / 2..size).map(move |y| (x, y))).collect(),

            28 => (0..size / 2).flat_map(|x| ((size / 2 + x)..size).map(move |y| (x, y))).collect(),
            29 => (0..size / 2).flat_map(|x| (0..(size / 2 - x)).map(move |y| (x, y))).collect(),
            30 => (size / 2..size).flat_map(|x| (0..(x - size / 2)).map(move |y| (x, y))).collect(),
            31 => (size / 2..size)
                .flat_map(|x| ((size + size / 2 - x)..size).map(move |y| (x, y)))
                .collect(),

            32 | 45 => (0..size).flat_map(|x| (0..(size / 2 + x).clamp(0, size)).map(move |y| (x, y))).collect(),
            33 | 46 => (0..size)
                .flat_map(|x| ((size / 2).saturating_sub(x)..size).map(move |y| (x, y)))
                .collect(),
            34 | 47 => (0..size).flat_map(|x| (x.saturating_sub(size / 2)..size).map(move |y| (x, y))).collect(),
            35 | 44 => (0..size)
                .flat_map(|x| (0..(size + size / 2 - x).clamp(0, size)).map(move |y| (x, y)))
                .collect(),
            other => unimplemented!("Shape {} is not implemented.", other),
        };

        Self { inner: points }
    }
}

impl IntoIterator for OverlayShape {
    type Item = (u32, u32);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

/// An iterator that given a shape and tilesize, yields which pixels should not be coloured.
///
/// Complements [`OverlayShape`].
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct UnderlayShape {
    inner: std::vec::IntoIter<(u32, u32)>,
}

impl UnderlayShape {
    /// Constructor for UnderlayShape.
    pub fn new(shape: Option<u8>, size: u32) -> Self {
        assert!(size.is_power_of_two(), "{} is an invalid size Only 2^n values are allowed.", size);

        let points: Vec<(u32, u32)> = match shape {
            None => (0..size).flat_map(|x| (0..size).map(move |y| (x, y))).collect(),
            Some(0) => vec![],

            Some(4) | Some(39) | Some(41) => (0..size).flat_map(|x| (0..x).map(move |y| (x, y))).collect(),
            Some(5) | Some(36) | Some(42) => (0..size).flat_map(|x| ((size - x)..size).map(move |y| (x, y))).collect(),
            Some(6) | Some(37) | Some(43) => (0..size).flat_map(|x| (x..size).map(move |y| (x, y))).collect(),
            Some(7) | Some(38) | Some(40) => (0..size).flat_map(|x| (0..(size - x)).map(move |y| (x, y))).collect(),

            Some(8) => (0..size)
                .flat_map(|x| ((size.saturating_sub(2 * x))..size).map(move |y| (x, y)))
                .collect(),
            Some(9) => (0..size).flat_map(|x| ((x / 2)..size).map(move |y| (x, y))).collect(),
            Some(10) => (0..size)
                .flat_map(|x| (0..(2 * (size - x)).clamp(0, size)).map(move |y| (x, y)))
                .collect(),

            Some(11) => (0..size).flat_map(|x| (0..((x + size) / 2)).map(move |y| (x, y))).collect(),
            Some(12) => (0..size)
                .flat_map(|x| ((2 * x).saturating_sub(size)..size).map(move |y| (x, y)))
                .collect(),
            Some(13) => (0..size).flat_map(|x| (0..(size - 1 - (x / 2))).map(move |y| (x, y))).collect(),
            Some(14) => (0..size).flat_map(|x| (0..(2 * x).clamp(0, size)).map(move |y| (x, y))).collect(),
            Some(15) => (0..size).flat_map(|x| (((size - x) / 2)..size).map(move |y| (x, y))).collect(),

            Some(16) => (0..(size / 2)).flat_map(|x| (0..(size - 2 * x)).map(move |y| (x, y))).collect(),
            Some(17) => (0..size).flat_map(|x| (0..(x / 2)).map(move |y| (x, y))).collect(),
            Some(18) => ((size / 2)..size)
                .flat_map(|x| ((size - 2 * (x - size / 2))..size).map(move |y| (x, y)))
                .collect(),
            Some(19) => (0..size).flat_map(|x| (((x + size) / 2)..size).map(move |y| (x, y))).collect(),
            Some(20) => ((size / 2)..size).flat_map(|x| (0..(2 * x - size)).map(move |y| (x, y))).collect(),
            Some(21) => (0..size).flat_map(|x| ((size - 1 - (x / 2))..size).map(move |y| (x, y))).collect(),
            Some(22) => (0..(size / 2)).flat_map(|x| ((2 * x)..size).map(move |y| (x, y))).collect(),
            Some(23) => (0..size).flat_map(|x| (0..((size - x) / 2)).map(move |y| (x, y))).collect(),

            Some(24) => (size / 2..size).flat_map(|x| (0..size).map(move |y| (x, y))).collect(),
            Some(25) => (0..size).flat_map(|x| (size / 2..size).map(move |y| (x, y))).collect(),
            Some(26) => (0..size / 2).flat_map(|x| (0..size).map(move |y| (x, y))).collect(),
            Some(27) => (0..size).flat_map(|x| (0..size / 2).map(move |y| (x, y))).collect(),

            Some(28) => (0..size).flat_map(|x| (0..(size / 2 + x).clamp(0, size)).map(move |y| (x, y))).collect(),
            Some(29) => (0..size)
                .flat_map(|x| ((size / 2).saturating_sub(x)..size).map(move |y| (x, y)))
                .collect(),
            Some(30) => (0..size).flat_map(|x| (x.saturating_sub(size / 2)..size).map(move |y| (x, y))).collect(),
            Some(31) => (0..size)
                .flat_map(|x| (0..(size + size / 2 - x).clamp(0, size)).map(move |y| (x, y)))
                .collect(),

            Some(32) | Some(45) => (0..size / 2).flat_map(|x| ((size / 2 + x)..size).map(move |y| (x, y))).collect(),
            Some(33) | Some(46) => (0..size / 2).flat_map(|x| (0..(size / 2 - x)).map(move |y| (x, y))).collect(),
            Some(34) | Some(47) => (size / 2..size).flat_map(|x| (0..(x - size / 2)).map(move |y| (x, y))).collect(),
            Some(35) | Some(44) => (size / 2..size)
                .flat_map(|x| ((size + size / 2 - x)..size).map(move |y| (x, y)))
                .collect(),
            Some(other) => unimplemented!("Shape {} is not implemented.", other),
        };

        Self { inner: points.into_iter() }
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
    use super::*;
    use std::collections::HashSet;

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
            let def: Option<u8> = None;
            for (x, y) in UnderlayShape::new(def, *size) {
                if !(x < *size && y < *size) {
                    panic!("{} {} {:?} {}", x, y, def, size)
                }
            }
            for shape in &shapes {
                for (x, y) in UnderlayShape::new(def, *size) {
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
