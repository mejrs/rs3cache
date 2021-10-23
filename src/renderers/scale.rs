use image::{GenericImageView, Pixel, Rgba, RgbaImage};
use itertools::{iproduct, izip};

/// Resizes a 512x512 image to 256x256

/// This function is used over the resize functions for two reasons:
/// - Performance, because image's resize function is shit.
/// - To avoid white lines around the edges.
pub fn resize_half(input: impl GenericImageView<Pixel = Rgba<u8>>) -> RgbaImage {
    assert_eq!(input.dimensions(), (512, 512));

    RgbaImage::from_fn(256, 256, |x, y| {
        let mut accum: [u16; 4] = [0, 0, 0, 0];

        // Iterate over 2x2 chunks and assign the average to the new pixel
        for (a, b) in iproduct!((x << 1)..((x << 1) + 2), (y << 1)..((y << 1) + 2)) {
            let pixel = input.get_pixel(a, b);
            for (accum_channel, pixel_channel) in izip!(accum.iter_mut(), pixel.channels()) {
                *accum_channel += *pixel_channel as u16;
            }
        }

        let accum = accum.map(|c| (c / 4) as u8);
        accum.into()
    })
}

/// Resizes a 1024x1024 image to 256x256

/// This function is used over the resize functions for two reasons:
/// - Performance, because image's resize function is shit.
/// - To avoid white lines around the edges.
pub fn resize_quarter(input: impl GenericImageView<Pixel = Rgba<u8>>) -> RgbaImage {
    assert_eq!(input.dimensions(), (1024, 1024));

    RgbaImage::from_fn(256, 256, |x, y| {
        let mut accum: [u16; 4] = [0, 0, 0, 0];

        // Iterate over 4x4 chunks and assign the average to the new pixel
        for (a, b) in iproduct!((x << 2)..((x << 2) + 4), (y << 2)..((y << 2) + 4)) {
            let pixel = input.get_pixel(a, b);
            for (accum_channel, pixel_channel) in izip!(accum.iter_mut(), pixel.channels()) {
                *accum_channel += *pixel_channel as u16;
            }
        }

        let accum = accum.map(|c| (c / 16) as u8);
        accum.into()
    })
}
