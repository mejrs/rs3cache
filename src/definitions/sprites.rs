use crate::{
    cache::{buf::Buffer, index::CacheIndex, indextype::IndexType},
    utils::{error::CacheResult, par::ParApply},
};
use image::{imageops, ImageBuffer, Rgba, RgbaImage};
use itertools::izip;
use std::{collections::HashMap, io::SeekFrom, iter};
/// Output directory for [save_all()]
const DIR: &str = "out/sprites";

/// Type alias for a rgba image.
pub type Sprite = ImageBuffer<Rgba<u8>, Vec<u8>>;

/// Saves an image of every sprite to disk.
pub fn save_all() -> CacheResult<()> {
    std::fs::create_dir_all(DIR)?;

    CacheIndex::new(IndexType::SPRITES)?.into_iter().par_apply(|mut archive| {
        debug_assert_eq!(archive.file_count(), 1);

        let file = archive
            .take_file(&0)
            .unwrap_or_else(|error| panic!("Unable to get file for sprite {}: {} ", archive.archive_id(), error));
        let images = deserialize(file).unwrap_or_else(|error| panic!("Error decoding sprite {}: {}", archive.archive_id(), error));
        images.into_iter().for_each(|(frame, img)| {
            let filename = format!("{}/{}-{}.png", DIR, archive.archive_id(), frame);
            img.save(&filename)
                .unwrap_or_else(|_| panic!("Unable to save sprite {}-{} to {}", archive.archive_id(), frame, filename));
        })
    });
    Ok(())
}

/// Returns a [`HashMap`] holding all sprites
///
/// Sprites are scaled according to `scale`, which may not be `0`.
///
/// # Errors
///
/// Raises [`CacheError`](crate::utils::error::CacheError) if any of `ids` does not correspond to a sprite.
///
/// Panics
/// **Panics** if `scale == 0`.

pub fn dumps(scale: u32, ids: Vec<u32>) -> CacheResult<HashMap<(u32, u32), Sprite>> {
    assert_ne!(scale, 0);

    let resizer = |(id, frames): (u32, HashMap<usize, Sprite>)| {
        frames.into_iter().map(move |(frame, img)| {
            let resized_img = imageops::resize(&img, img.width() * scale, img.height() * scale, imageops::Nearest);
            ((id, frame as u32), resized_img)
        })
    };

    let sprites = CacheIndex::new(IndexType::SPRITES)?
        .retain(ids)
        .into_iter()
        .map(|mut archive| archive.take_file(&0).and_then(deserialize).map(|frames| (archive.archive_id(), frames)))
        .collect::<CacheResult<Vec<(u32, _)>>>()?
        .into_iter()
        .flat_map(resizer)
        .collect::<HashMap<(u32, u32), Sprite>>();
    Ok(sprites)
}

fn deserialize(file: Vec<u8>) -> CacheResult<HashMap<usize, Sprite>> {
    let mut buffer = Buffer::new(file);
    buffer.seek(SeekFrom::End(-2))?;

    let data = buffer.read_unsigned_short();
    let format = data >> 15;
    let count = (data & 0x7FFF) as usize;

    let imgs = match format {
        0 => {
            buffer.seek(SeekFrom::End(-7 - (count as i64) * 8))?;

            let _big_width = buffer.read_unsigned_short();
            let _big_height = buffer.read_unsigned_short();
            let palette_count = buffer.read_unsigned_byte() as usize;

            let _min_xs = iter::repeat_with(|| buffer.read_unsigned_short()).take(count).collect::<Vec<_>>();
            let _min_ys = iter::repeat_with(|| buffer.read_unsigned_short()).take(count).collect::<Vec<_>>();
            let widths = iter::repeat_with(|| buffer.read_unsigned_short()).take(count).collect::<Vec<_>>();
            let heights = iter::repeat_with(|| buffer.read_unsigned_short()).take(count).collect::<Vec<_>>();

            let pos = -7 - (count as i64) * 8 - (palette_count as i64) * 3;

            buffer.seek(SeekFrom::End(pos))?;

            let palette = iter::repeat_with(|| buffer.read_rgb()).take(palette_count).collect::<Vec<_>>();

            buffer.seek(SeekFrom::Start(0))?;

            izip!(0..count, widths, heights)
                .filter_map(|(index, width, height)| {
                    let pixel_count = width as usize * height as usize;
                    let [transposed, alpha, ..] = buffer.read_bitflags();
                    if pixel_count != 0 {
                        let base = buffer.read_n_bytes(pixel_count);

                        let mask = if alpha {
                            buffer.read_n_bytes(pixel_count)
                        } else {
                            iter::repeat(255u8).take(pixel_count).collect::<Vec<_>>()
                        };
                        let mut img = if !transposed {
                            RgbaImage::new(width as u32, height as u32)
                        } else {
                            RgbaImage::new(height as u32, width as u32)
                        };

                        img.pixels_mut().zip(base).zip(mask).for_each(|((pixel, idx), alpha_channel)| {
                            let ((red, green, blue), alpha) = if idx == 0 {
                                ((255, 0, 255), 0)
                            } else {
                                (palette[idx as usize - 1], alpha_channel)
                            };

                            pixel[0] = red;
                            pixel[1] = green;
                            pixel[2] = blue;
                            pixel[3] = alpha;
                        });

                        if transposed {
                            img = imageops::rotate90(&imageops::flip_vertical(&img));
                        }

                        Some((index, img))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>()
        }
        1 => {
            buffer.seek(SeekFrom::Start(0))?;
            let ty = buffer.read_unsigned_byte();
            assert_eq!(ty, 0, "Unknown image type.");

            let [alpha, ..] = buffer.read_bitflags();
            let width = buffer.read_unsigned_short();
            let height = buffer.read_unsigned_short();
            let pixel_count = width as usize * height as usize;

            let base = iter::repeat_with(|| buffer.read_rgb()).take(pixel_count).collect::<Vec<_>>();

            let mask = if alpha {
                buffer.read_n_bytes(pixel_count)
            } else {
                vec![255_u8; pixel_count]
            };

            let mut img = RgbaImage::new(width as u32, height as u32);

            img.pixels_mut().zip(base).zip(mask).for_each(|((pixel, rgb), alpha)| {
                let (red, green, blue) = rgb;
                pixel[0] = red;
                pixel[1] = green;
                pixel[2] = blue;
                pixel[3] = alpha;
            });

            let mut images = HashMap::with_capacity(1);
            images.insert(0_usize, img);

            images
        }
        _ => unimplemented!("Unknown sprite format..."),
    };
    Ok(imgs)
}

#[cfg(test)]
mod sprite_tests {
    use super::*;

    #[test]
    fn render_some_0() -> CacheResult<()> {
        fn dump(id: u32, frame: u32) -> CacheResult<Sprite> {
            let mut archive = CacheIndex::new(IndexType::SPRITES)?.archive(id)?;
            let file = archive.take_file(&0)?;
            assert!(file.len() != 0, "{:?}", file);
            let mut images = deserialize(file)?;
            Ok(images.remove(&(frame as usize))?)
        }

        std::fs::create_dir_all("tests/sprites/method_0".to_string())?;

        for id in vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 694, 14659, 3034] {
            let frame = 0;
            let sprite = dump(id, frame)?;
            let filename = format!("tests/sprites/method_0/{}-{}.png", id, frame);
            sprite.save(filename).expect("Error saving image");
        }
        Ok(())
    }

    #[test]
    fn render_some_1() -> CacheResult<()> {
        let ids = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 694, 14659, 3034];

        let sprites = dumps(2, ids)?;
        println!("{:?}", sprites.keys().collect::<Vec<_>>());

        Ok(())
    }
    #[test]
    #[should_panic]
    fn render_nonexistant() {
        let ids = vec![40000, 50000];

        let sprites = dumps(2, ids).expect("should be unable to create a limited archiveiterator if the key is not in metadatas");

        println!("Should have not been able to deserialize these: {:?}", sprites.keys().collect::<Vec<_>>());
    }
}
