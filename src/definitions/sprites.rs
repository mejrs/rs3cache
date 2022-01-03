use std::{
    collections::BTreeMap,
    io::{Cursor, Seek, SeekFrom},
    iter,
};

use bytes::{Buf, Bytes};
use fstrings::{f, format_args_f};
use image::{imageops, ImageBuffer, Rgba, RgbaImage};
use itertools::izip;
use path_macro::path;
use rayon::iter::{ParallelBridge, ParallelIterator};
use rs3cache_core::buf::BufExtra;

use crate::cache::{error::CacheResult, index::CacheIndex, indextype::IndexType};

/// Type alias for a rgba image.
pub type Sprite = ImageBuffer<Rgba<u8>, Vec<u8>>;

/// Saves an image of every sprite to disk.
#[cfg(any(feature = "rs3", feature = "osrs"))]
pub fn save_all(config: &crate::cli::Config) -> CacheResult<()> {
    std::fs::create_dir_all(path!(config.output / "sprites"))?;

    let index = CacheIndex::new(IndexType::SPRITES, &config.input)?;

    #[cfg(feature = "rs3")]
    let versions: BTreeMap<u32, ::filetime::FileTime> = index
        .metadatas()
        .iter()
        .map(|(_, meta)| (meta.archive_id(), ::filetime::FileTime::from_unix_time(meta.version() as i64, 0)))
        .collect();

    index.into_iter().par_bridge().for_each(|archive| {
        let archive = archive.unwrap();
        debug_assert_eq!(archive.file_count(), 1);

        let file = archive
            .file(&0)
            .unwrap_or_else(|error| panic!("Unable to get file for sprite {}: {} ", archive.archive_id(), error));
        let images = deserialize(file).unwrap_or_else(|error| panic!("Error decoding sprite {}: {}", archive.archive_id(), error));
        images.into_iter().for_each(|(frame, img)| {
            let id = archive.archive_id();
            let filename = path!(config.output / "sprites" / f!("{id}-{frame}.png"));
            img.save(&filename)
                .unwrap_or_else(|_| panic!("Unable to save sprite {}-{} to {}", id, frame, filename.to_string_lossy()));

            #[cfg(feature = "rs3")]
            {
                let file = ::std::fs::OpenOptions::new().write(true).open(&filename).unwrap();

                let date = versions[&id];

                ::filetime::set_file_handle_times(&file, Some(date), Some(date)).unwrap();
            }
        })
    });
    Ok(())
}

#[derive(Debug)]
struct IndexEntry {
    max_width: u16,
    max_height: u16,
    colour_count: u8,
    palette: Vec<[u8; 3]>,
}

impl IndexEntry {
    fn deserialize(buffer: &mut Bytes) -> Self {
        let max_width = buffer.get_u16();
        let max_height = buffer.get_u16();
        let colour_count = buffer.get_u8().checked_sub(1).unwrap();
        let palette = iter::repeat_with(|| buffer.get_rgb()).take(colour_count as usize).collect();
        Self {
            max_width,
            max_height,
            colour_count,
            palette,
        }
    }
}

#[derive(Debug)]
struct Entry {
    offset_x: u8,
    offset_y: u8,
    width: u16,
    height: u16,
    transposed: u8,
}

impl Entry {
    fn deserialize(buffer: &mut Bytes) -> Self {
        Self {
            offset_x: buffer.get_u8(),
            offset_y: buffer.get_u8(),
            width: buffer.get_u16(),
            height: buffer.get_u16(),
            transposed: buffer.get_u8(),
        }
    }
}

#[cfg(feature = "legacy")]
fn make_image(index_entry: &IndexEntry, entry: &Entry, data: Bytes) -> Sprite {
    let mut image = RgbaImage::new(entry.width as u32, entry.height as u32);
    image.pixels_mut().zip(&*data).for_each(|(pixel, &idx)| {
        let ([red, green, blue], alpha) = if idx == 0 {
            ([255, 0, 255], 0)
        } else {
            (index_entry.palette[idx.checked_sub(1).unwrap() as usize], 255)
        };

        pixel[0] = red;
        pixel[1] = green;
        pixel[2] = blue;
        pixel[3] = alpha;
    });
    if entry.transposed == 1 {
        image = imageops::rotate90(&imageops::flip_vertical(&image));
    }
    image
}

#[cfg(feature = "legacy")]
pub fn save_all(config: &crate::cli::Config) -> CacheResult<()> {
    use rs3cache_core::hash::hash_archive;

    std::fs::create_dir_all(path!(config.output / "sprites"))?;

    let index = CacheIndex::new(0, &config.input)?;
    let mut files = index.archive(4)?.take_files_named();

    let meta = files.remove(&hash_archive("index.dat")).unwrap();
    let mut data = files.remove(&hash_archive("mapscene.dat")).unwrap();
    let offset = data.get_u16();
    let mut entry_data = meta.slice((offset as usize)..);
    let index_entry = IndexEntry::deserialize(&mut entry_data);

    let mut id = 0;
    while data.has_remaining() {
        let entry = Entry::deserialize(&mut entry_data);
        let this_data = data.split_to((entry.width * entry.height) as usize);
        let img = make_image(&index_entry, &entry, this_data);
        let filename = path!(config.output / "sprites" / f!("mapscene-{id}.png"));
        img.save(&filename).unwrap();

        id += 1;
    }

    Ok(())
}

#[cfg(feature = "legacy")]
pub fn get_mapscenes(scale: u32, config: &crate::cli::Config) -> CacheResult<BTreeMap<(u32, u32), Sprite>> {
    use rs3cache_core::hash::hash_archive;

    std::fs::create_dir_all(path!(config.output / "sprites"))?;

    let index = CacheIndex::new(0, &config.input)?;
    let mut files = index.archive(4)?.take_files_named();

    let meta = files.remove(&hash_archive("index.dat")).unwrap();
    let mut data = files.remove(&hash_archive("mapscene.dat")).unwrap();
    let offset = data.get_u16();
    let mut entry_data = meta.slice((offset as usize)..);
    let index_entry = IndexEntry::deserialize(&mut entry_data);

    let mut out = BTreeMap::new();

    let mut frame = 0;
    while data.has_remaining() {
        let entry = Entry::deserialize(&mut entry_data);
        let this_data = data.split_to((entry.width * entry.height) as usize);
        let img = make_image(&index_entry, &entry, this_data);
        let img = imageops::resize(&img, img.width() * scale, img.height() * scale, imageops::Nearest);
        out.insert((317, frame), img);
        frame += 1;
    }
    Ok(out)
}

/// Returns a [`BTreeMap`] holding all sprites in `ids`.
///
/// Sprites are scaled according to `scale`, which may not be `0`.
///
/// # Errors
///
/// Raises [`CacheError`](rs3cache_core::error::CacheError) if any of `ids` does not correspond to a sprite.
///
/// # Panics
///
/// **Panics** if `scale == 0`.

pub fn dumps(scale: u32, ids: Vec<u32>, config: &crate::cli::Config) -> CacheResult<BTreeMap<(u32, u32), Sprite>> {
    assert_ne!(scale, 0);

    let resizer = |(id, frames): (u32, BTreeMap<usize, Sprite>)| {
        frames.into_iter().map(move |(frame, img)| {
            let resized_img = imageops::resize(&img, img.width() * scale, img.height() * scale, imageops::Nearest);
            ((id, frame as u32), resized_img)
        })
    };

    let sprites = CacheIndex::new(IndexType::SPRITES, &config.input)?
        .retain(ids)
        .into_iter()
        .map(Result::unwrap)
        .map(|archive| archive.file(&0).and_then(deserialize).map(|frames| (archive.archive_id(), frames)))
        .collect::<CacheResult<Vec<(u32, _)>>>()?
        .into_iter()
        .flat_map(resizer)
        .collect::<BTreeMap<(u32, u32), Sprite>>();
    Ok(sprites)
}

pub fn deserialize(buffer: Bytes) -> CacheResult<BTreeMap<usize, Sprite>> {
    let mut buffer = Cursor::new(buffer);

    buffer.seek(SeekFrom::End(-2))?;

    let data = buffer.get_u16();
    let format = data >> 15;
    let count = (data & 0x7FFF) as usize;
    dbg!(count);
    //let count = 4;

    let imgs = match format {
        0 => {
            buffer.seek(SeekFrom::End(-7 - (count as i64) * 8))?;

            let _big_width = buffer.get_u16();
            let _big_height = buffer.get_u16();
            let palette_count = buffer.get_u8() as usize;

            let _min_xs = iter::repeat_with(|| buffer.get_u16()).take(count).collect::<Vec<_>>();
            let _min_ys = iter::repeat_with(|| buffer.get_u16()).take(count).collect::<Vec<_>>();
            let widths = iter::repeat_with(|| buffer.get_u16()).take(count).collect::<Vec<_>>();
            let heights = iter::repeat_with(|| buffer.get_u16()).take(count).collect::<Vec<_>>();

            let pos = -7 - (count as i64) * 8 - (palette_count as i64) * 3;

            buffer.seek(SeekFrom::End(pos))?;

            let palette = iter::repeat_with(|| buffer.get_rgb()).take(palette_count).collect::<Vec<_>>();

            buffer.seek(SeekFrom::Start(0))?;

            izip!(0..count, widths, heights)
                .filter_map(|(index, width, height)| {
                    let pixel_count = width as usize * height as usize;
                    let [transposed, alpha, ..] = buffer.get_bitflags();
                    if pixel_count != 0 {
                        let base = buffer.copy_to_bytes(pixel_count);

                        let mask = if alpha {
                            buffer.copy_to_bytes(pixel_count)
                        } else {
                            vec![255_u8; pixel_count].into()
                        };
                        let mut img = if !transposed {
                            RgbaImage::new(width as u32, height as u32)
                        } else {
                            RgbaImage::new(height as u32, width as u32)
                        };

                        img.pixels_mut().zip(base).zip(mask).for_each(|((pixel, idx), alpha_channel)| {
                            let ([red, green, blue], alpha) = if idx == 0 {
                                ([255, 0, 255], 0)
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
                .collect::<BTreeMap<_, _>>()
        }
        1 => {
            buffer.seek(SeekFrom::Start(0))?;
            let ty = buffer.get_u8();
            assert_eq!(ty, 0, "Unknown image type.");

            let [alpha, ..] = buffer.get_bitflags();
            let width = buffer.get_u16();
            let height = buffer.get_u16();
            let pixel_count = width as usize * height as usize;

            let base = iter::repeat_with(|| buffer.get_rgb()).take(pixel_count).collect::<Vec<_>>();

            let mask = if alpha {
                buffer.copy_to_bytes(pixel_count)
            } else {
                vec![255_u8; pixel_count].into()
            };

            let mut img = RgbaImage::new(width as u32, height as u32);

            img.pixels_mut().zip(base).zip(mask).for_each(|((pixel, rgb), alpha)| {
                let [red, green, blue] = rgb;
                pixel[0] = red;
                pixel[1] = green;
                pixel[2] = blue;
                pixel[3] = alpha;
            });

            let mut images = BTreeMap::new();
            images.insert(0_usize, img);

            images
        }
        _ => unimplemented!("Unknown sprite format..."),
    };
    Ok(imgs)
}

#[cfg(test)]
#[cfg(any(feature = "rs3", feature = "osrs"))]
mod sprite_tests {
    use super::*;

    #[test]
    fn render_some_0() -> CacheResult<()> {
        fn dump(id: u32, frame: u32) -> CacheResult<Sprite> {
            let config = crate::cli::Config::env();

            let archive = CacheIndex::new(IndexType::SPRITES, &config.input)?.archive(id)?;
            let file = archive.file(&0)?;
            assert!(file.len() != 0, "{:?}", file);
            let mut images = deserialize(file).unwrap();
            Ok(images.remove(&(frame as usize)).unwrap())
        }

        std::fs::create_dir_all("test_data/sprites/method_0".to_string())?;

        for id in vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 694, 3034] {
            let frame = 0;
            let sprite = dump(id, frame)?;
            let filename = format!("test_data/sprites/method_0/{}-{}.png", id, frame);
            sprite.save(filename).expect("Error saving image");
        }
        Ok(())
    }

    #[test]
    fn render_some_1() -> CacheResult<()> {
        let config = crate::cli::Config::env();

        let ids = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 694, 3034];

        let sprites = dumps(2, ids, &config)?;
        println!("{:?}", sprites.keys().collect::<Vec<_>>());

        Ok(())
    }
    #[test]
    #[should_panic]
    fn render_nonexistant() {
        let config = crate::cli::Config::env();

        let ids = vec![40000, 50000];

        let sprites = dumps(2, ids, &config).expect("should be unable to create a limited archiveiterator if the key is not in metadatas");

        println!("Should have not been able to deserialize these: {:?}", sprites.keys().collect::<Vec<_>>());
    }
}
