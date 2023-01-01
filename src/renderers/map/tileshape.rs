pub fn draw_overlay(shape: u8, size: u32, fun: impl FnMut((u32, u32))) {
    debug_assert!(size.is_power_of_two(), "{size} is an invalid size, only 2^n values are allowed.");

    match shape {
        0 => (0..size).flat_map(move |x| (0..size).map(move |y| (x, y))).for_each(fun),

        4 | 39 | 41 => (0..size).flat_map(move |x| (x..size).map(move |y| (x, y))).for_each(fun),
        5 | 36 | 42 => (0..size).flat_map(move |x| (0..(size - x)).map(move |y| (x, y))).for_each(fun),
        6 | 37 | 43 => (0..size).flat_map(move |x| (0..x).map(move |y| (x, y))).for_each(fun),
        7 | 38 | 40 => (0..size).flat_map(move |x| ((size - x)..size).map(move |y| (x, y))).for_each(fun),

        8 => (0..(size / 2)).flat_map(move |x| (0..(size - 2 * x)).map(move |y| (x, y))).for_each(fun),
        9 => (0..size).flat_map(move |x| (0..(x / 2)).map(move |y| (x, y))).for_each(fun),
        10 => ((size / 2)..size)
            .flat_map(move |x| ((size - 2 * (x - size / 2))..size).map(move |y| (x, y)))
            .for_each(fun),
        11 => (0..size).flat_map(move |x| (((x + size) / 2)..size).map(move |y| (x, y))).for_each(fun),
        12 => ((size / 2)..size)
            .flat_map(move |x| (0..(2 * x - size)).map(move |y| (x, y)))
            .for_each(fun),
        13 => (0..size)
            .flat_map(move |x| ((size - 1 - (x / 2))..size).map(move |y| (x, y)))
            .for_each(fun),
        14 => (0..(size / 2)).flat_map(move |x| ((2 * x)..size).map(move |y| (x, y))).for_each(fun),
        15 => (0..size).flat_map(move |x| (0..((size - x) / 2)).map(move |y| (x, y))).for_each(fun),

        16 => (0..size)
            .flat_map(move |x| ((size.saturating_sub(2 * x))..size).map(move |y| (x, y)))
            .for_each(fun),
        17 => (0..size).flat_map(move |x| ((x / 2)..size).map(move |y| (x, y))).for_each(fun),
        18 => (0..size)
            .flat_map(move |x| (0..(2 * (size - x)).clamp(0, size)).map(move |y| (x, y)))
            .for_each(fun),
        19 => (0..size).flat_map(move |x| (0..((x + size) / 2)).map(move |y| (x, y))).for_each(fun),
        20 => (0..size)
            .flat_map(move |x| ((2 * x).saturating_sub(size)..size).map(move |y| (x, y)))
            .for_each(fun),
        21 => (0..size).flat_map(move |x| (0..(size - 1 - (x / 2))).map(move |y| (x, y))).for_each(fun),
        22 => (0..size)
            .flat_map(move |x| (0..(2 * x).clamp(0, size)).map(move |y| (x, y)))
            .for_each(fun),
        23 => (0..size).flat_map(move |x| (((size - x) / 2)..size).map(move |y| (x, y))).for_each(fun),

        24 => (0..size / 2).flat_map(move |x| (0..size).map(move |y| (x, y))).for_each(fun),
        25 => (0..size).flat_map(move |x| (0..size / 2).map(move |y| (x, y))).for_each(fun),
        26 => (size / 2..size).flat_map(move |x| (0..size).map(move |y| (x, y))).for_each(fun),
        27 => (0..size).flat_map(move |x| (size / 2..size).map(move |y| (x, y))).for_each(fun),

        28 => (0..size / 2).flat_map(move |x| ((size / 2 + x)..size).map(move |y| (x, y))).for_each(fun),
        29 => (0..size / 2).flat_map(move |x| (0..(size / 2 - x)).map(move |y| (x, y))).for_each(fun),
        30 => (size / 2..size).flat_map(move |x| (0..(x - size / 2)).map(move |y| (x, y))).for_each(fun),
        31 => (size / 2..size)
            .flat_map(move |x| ((size + size / 2 - x)..size).map(move |y| (x, y)))
            .for_each(fun),
        32 | 45 => (0..size)
            .flat_map(move |x| (0..(size / 2 + x).clamp(0, size)).map(move |y| (x, y)))
            .for_each(fun),
        33 | 46 => (0..size)
            .flat_map(move |x| ((size / 2).saturating_sub(x)..size).map(move |y| (x, y)))
            .for_each(fun),
        34 | 47 => (0..size)
            .flat_map(move |x| (x.saturating_sub(size / 2)..size).map(move |y| (x, y)))
            .for_each(fun),
        35 | 44 => (0..size)
            .flat_map(move |x| (0..(size + size / 2 - x).clamp(0, size)).map(move |y| (x, y)))
            .for_each(fun),
        other => unimplemented!("Shape {} is not implemented.", other),
    };
}

pub fn draw_underlay(shape: Option<u8>, size: u32, fun: impl FnMut((u32, u32))) {
    debug_assert!(size.is_power_of_two(), "{size} is an invalid size, only 2^n values are allowed.");

    match shape {
        None => (0..size).flat_map(move |x| (0..size).map(move |y| (x, y))).for_each(fun),
        Some(0) => { /* draw nothing */ }

        Some(4) | Some(39) | Some(41) => (0..size).flat_map(move |x| (0..x).map(move |y| (x, y))).for_each(fun),
        Some(5) | Some(36) | Some(42) => (0..size).flat_map(move |x| ((size - x)..size).map(move |y| (x, y))).for_each(fun),
        Some(6) | Some(37) | Some(43) => (0..size).flat_map(move |x| (x..size).map(move |y| (x, y))).for_each(fun),
        Some(7) | Some(38) | Some(40) => (0..size).flat_map(move |x| (0..(size - x)).map(move |y| (x, y))).for_each(fun),

        Some(8) => (0..size)
            .flat_map(move |x| ((size.saturating_sub(2 * x))..size).map(move |y| (x, y)))
            .for_each(fun),
        Some(9) => (0..size).flat_map(move |x| ((x / 2)..size).map(move |y| (x, y))).for_each(fun),
        Some(10) => (0..size)
            .flat_map(move |x| (0..(2 * (size - x)).clamp(0, size)).map(move |y| (x, y)))
            .for_each(fun),
        Some(11) => (0..size).flat_map(move |x| (0..((x + size) / 2)).map(move |y| (x, y))).for_each(fun),
        Some(12) => (0..size)
            .flat_map(move |x| ((2 * x).saturating_sub(size)..size).map(move |y| (x, y)))
            .for_each(fun),
        Some(13) => (0..size).flat_map(move |x| (0..(size - 1 - (x / 2))).map(move |y| (x, y))).for_each(fun),
        Some(14) => (0..size)
            .flat_map(move |x| (0..(2 * x).clamp(0, size)).map(move |y| (x, y)))
            .for_each(fun),
        Some(15) => (0..size).flat_map(move |x| (((size - x) / 2)..size).map(move |y| (x, y))).for_each(fun),

        Some(16) => (0..(size / 2)).flat_map(move |x| (0..(size - 2 * x)).map(move |y| (x, y))).for_each(fun),
        Some(17) => (0..size).flat_map(move |x| (0..(x / 2)).map(move |y| (x, y))).for_each(fun),
        Some(18) => ((size / 2)..size)
            .flat_map(move |x| ((size - 2 * (x - size / 2))..size).map(move |y| (x, y)))
            .for_each(fun),
        Some(19) => (0..size).flat_map(move |x| (((x + size) / 2)..size).map(move |y| (x, y))).for_each(fun),
        Some(20) => ((size / 2)..size)
            .flat_map(move |x| (0..(2 * x - size)).map(move |y| (x, y)))
            .for_each(fun),
        Some(21) => (0..size)
            .flat_map(move |x| ((size - 1 - (x / 2))..size).map(move |y| (x, y)))
            .for_each(fun),
        Some(22) => (0..(size / 2)).flat_map(move |x| ((2 * x)..size).map(move |y| (x, y))).for_each(fun),
        Some(23) => (0..size).flat_map(move |x| (0..((size - x) / 2)).map(move |y| (x, y))).for_each(fun),

        Some(24) => (size / 2..size).flat_map(move |x| (0..size).map(move |y| (x, y))).for_each(fun),
        Some(25) => (0..size).flat_map(move |x| (size / 2..size).map(move |y| (x, y))).for_each(fun),
        Some(26) => (0..size / 2).flat_map(move |x| (0..size).map(move |y| (x, y))).for_each(fun),
        Some(27) => (0..size).flat_map(move |x| (0..size / 2).map(move |y| (x, y))).for_each(fun),

        Some(28) => (0..size)
            .flat_map(move |x| (0..(size / 2 + x).clamp(0, size)).map(move |y| (x, y)))
            .for_each(fun),
        Some(29) => (0..size)
            .flat_map(move |x| ((size / 2).saturating_sub(x)..size).map(move |y| (x, y)))
            .for_each(fun),
        Some(30) => (0..size)
            .flat_map(move |x| (x.saturating_sub(size / 2)..size).map(move |y| (x, y)))
            .for_each(fun),
        Some(31) => (0..size)
            .flat_map(move |x| (0..(size + size / 2 - x).clamp(0, size)).map(move |y| (x, y)))
            .for_each(fun),
        Some(32) | Some(45) => (0..size / 2).flat_map(move |x| ((size / 2 + x)..size).map(move |y| (x, y))).for_each(fun),
        Some(33) | Some(46) => (0..size / 2).flat_map(move |x| (0..(size / 2 - x)).map(move |y| (x, y))).for_each(fun),
        Some(34) | Some(47) => (size / 2..size).flat_map(move |x| (0..(x - size / 2)).map(move |y| (x, y))).for_each(fun),
        Some(35) | Some(44) => (size / 2..size)
            .flat_map(move |x| ((size + size / 2 - x)..size).map(move |y| (x, y)))
            .for_each(fun),
        Some(other) => unimplemented!("Shape {} is not implemented.", other),
    };
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
                draw_overlay(*shape, *size, |(x, y)| {
                    if !(x < *size && y < *size) {
                        panic!("{x} {y} {shape} {size}")
                    }
                });
            }
        }
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
            draw_underlay(None, *size, |(x, y)| {
                if !(x < *size && y < *size) {
                    panic!("{x} {y} {size}")
                }
            });

            for shape in &shapes {
                draw_underlay(Some(*shape), *size, |(x, y)| {
                    if !(x < *size && y < *size) {
                        panic!("{x} {y} {size}")
                    }
                });
            }
        }
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
                let mut collection = Vec::new();

                draw_underlay(Some(*shape), *size, |(x, y)| {
                    collection.push((x, y));
                });

                draw_overlay(*shape, *size, |(x, y)| {
                    collection.push((x, y));
                });

                assert_eq!(collection.len() as u32, size * size);

                let mut uniq = HashSet::new();
                assert!(collection.into_iter().all(move |x| uniq.insert(x)));
            }
        }
    }
}
