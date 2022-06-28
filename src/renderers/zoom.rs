use std::{collections::HashSet, ffi::OsString, fs, io, sync::LazyLock, ops::Range, path::Path};

use image::{imageops, io::Reader as ImageReader, ImageBuffer, ImageError, ImageFormat, Rgba, RgbaImage};
use indicatif::ProgressIterator;
use itertools::{iproduct, izip};
use path_macro::path;
use rayon::iter::{ParallelBridge, ParallelIterator};
use regex::Regex;

use crate::{cache::error::CacheResult, renderers::scale};

static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?P<p>\d+)(?:_)(?P<i>\d+)(?:_)(?P<j>\d+)(?:\.png)").expect("Regex is cursed."));

/// Given a folder and a range of zoom levels, recursively creates tiles for all zoom levels.
pub fn render_zoom_levels(folder: impl AsRef<Path> + Send + Sync, mapid: i32, range: Range<i8>, backfill: [u8; 4]) -> CacheResult<()> {
    let zoom_levels = range.rev();
    for zoom in zoom_levels {
        fs::create_dir_all(path!(folder / format!("{mapid}/{zoom}")))?;

        let new_tile_coordinates = get_future_filenames(&folder, mapid, zoom + 1)?.into_iter();

        let func = |(p, i, j)| {
            let img = make_tile(&folder, mapid, zoom, p, i, j, backfill).unwrap();
            let filename = path!(folder / format!("{mapid}/{zoom}/{p}_{i}_{j}.png"));
            img.save(filename).unwrap();
        };

        new_tile_coordinates.progress().par_bridge().for_each(func);
    }
    Ok(())
}

fn to_coordinates(text: OsString) -> CacheResult<(i32, i32, i32)> {
    let caps = RE.captures(text.to_str().unwrap()).unwrap();
    let p = caps.name("p").unwrap().as_str().parse::<i32>()?;
    let i = caps.name("i").unwrap().as_str().parse::<i32>()?;
    let j = caps.name("j").unwrap().as_str().parse::<i32>()?;
    Ok((p, i, j))
}

fn make_tile(
    folder: impl AsRef<Path>,
    mapid: i32,
    target_zoom: i8,
    target_plane: i32,
    target_i: i32,
    target_j: i32,
    backfill: [u8; 4],
) -> CacheResult<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut base = RgbaImage::from_fn(512, 512, |_, _| Rgba(backfill));

    let files = get_files(folder, mapid, target_zoom, target_plane, target_i, target_j);

    for ((di, dj), img) in files {
        match img {
            Ok(f) => {
                let img = f.into_rgba8();
                imageops::overlay(&mut base, &img, (256 * di) as i64, 256 * (1 - dj) as i64);
            }
            // can be missing; if so, swallow
            Err(ImageError::IoError(ref e)) if e.kind() == io::ErrorKind::NotFound => {}
            Err(other_error) => return Err(other_error.into()),
        }
    }
    let scaled = scale::resize_half(base);
    Ok(scaled)
}

fn get_files(
    folder: impl AsRef<Path>,
    mapid: i32,
    target_zoom: i8,
    target_plane: i32,
    target_i: i32,
    target_j: i32,
) -> [((i32, i32), Result<image::DynamicImage, ImageError>); 4] {
    [(0, 0), (0, 1), (1, 0), (1, 1)].map(|(di, dj)| {
        let i = (target_i << 1) + di;
        let j = (target_j << 1) + dj;
        let zoom = target_zoom + 1;
        let filename = path!(folder / format!("{mapid}/{zoom}/{target_plane}_{i}_{j}.png"));
        ((di, dj), image::open(filename))
    })
}

fn get_future_filenames(folder: impl AsRef<Path>, mapid: i32, zoom: i8) -> CacheResult<HashSet<(i32, i32, i32)>> {
    let dir = path!(folder / format!("{mapid}/{zoom}"));

    let new_tiles = fs::read_dir(&dir)?
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .map(|entry| entry.file_name())
        .map(to_coordinates)
        .collect::<CacheResult<Vec<_>>>()?
        .into_iter()
        .map(|(p, i, j)| (p, i >> 1, j >> 1))
        .collect::<HashSet<_>>();

    Ok(new_tiles)
}
