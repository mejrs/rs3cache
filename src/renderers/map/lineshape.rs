pub fn draw(ty: u8, rotation: u8, size: u32, fun: impl FnMut((u32, u32))) {
    debug_assert!(size.is_power_of_two(), "{} is an invalid size Only 2^n values are allowed.", size);

    match (ty, rotation) {
        (0, 0) => (0..(size / 4)).flat_map(|x| (0..size).map(move |y| (x, y))).for_each(fun),
        (0, 1) => (0..size).flat_map(|x| (0..(size / 4)).map(move |y| (x, y))).for_each(fun),
        (0, 2) => ((size * 3 / 4)..size).flat_map(|x| (0..size).map(move |y| (x, y))).for_each(fun),
        (0, 3) => (0..size).flat_map(|x| ((size * 3 / 4)..size).map(move |y| (x, y))).for_each(fun),
        (2, 0) => (0..size)
            .flat_map(|x| (0..if x < size / 4 { size } else { size / 4 }).map(move |y| (x, y)))
            .for_each(fun),
        (2, 1) => (0..size)
            .flat_map(|x| (0..if x < size * 3 / 4 { size / 4 } else { size }).map(move |y| (x, y)))
            .for_each(fun),
        (2, 2) => (0..size)
            .flat_map(|x| (if x < size * 3 / 4 { size * 3 / 4 } else { 0 }..size).map(move |y| (x, y)))
            .for_each(fun),
        (2, 3) => (0..size)
            .flat_map(|x| (if x < size / 4 { 0 } else { size * 3 / 4 }..size).map(move |y| (x, y)))
            .for_each(fun),
        (9, 0) | (9, 2) => (0..size)
            .flat_map(|x| ((size - x).saturating_sub(size / 8)..(size - x + size / 8).clamp(0, size)).map(move |y| (x, y)))
            .for_each(fun),
        (9, 1) | (9, 3) => (0..size)
            .flat_map(|x| (x.saturating_sub(size / 8)..(x + size / 8).clamp(0, size)).map(move |y| (x, y)))
            .for_each(fun),

        (other_type, other_rot) => unimplemented!("LineShape for type {} with rotation {} is not implemented.", other_type, other_rot),
    };
}

#[cfg(test)]
mod line_shape_tests {
    use super::LineShape;

    // Unsafe code in renderers/map.rs depends on this test passing
    #[test]
    fn test_bounds() {
        let types = [0, 2, 9];
        let rotations = [0, 1, 2, 3];
        let sizes = [8, 16, 32, 64, 128, 256];
        for ty in &types {
            for rot in &rotations {
                for size in &sizes {
                    for (x, y) in LineShape::new(*ty, *rot, *size) {
                        if !(x < *size && y < *size) {
                            panic!("{} {} {} {} {}", x, y, ty, rot, size)
                        }
                    }
                }
            }
        }
    }

    #[test]
    #[should_panic]
    fn unpower_of_two() {
        let _ = LineShape::new(0, 0, 7);
    }

    #[test]
    #[should_panic]
    fn invalid_type() {
        let _ = LineShape::new(4, 0, 8);
    }
}
