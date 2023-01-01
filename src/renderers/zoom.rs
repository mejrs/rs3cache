use std::{collections::HashSet, ffi::OsString, fs, io, ops::Range, path::Path, sync::LazyLock};

use image::{imageops, io::Reader as ImageReader, ImageBuffer, ImageError, ImageFormat, Rgba, RgbaImage};
use indicatif::ProgressIterator;
use itertools::{iproduct, izip};
use path_macro::path;
use rayon::iter::{ParallelBridge, ParallelIterator};
use regex::Regex;
use rs3cache_backend::error::CacheError;

use crate::{cache::error::CacheResult, renderers::scale};

static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?P<p>\d+)(?:_)(?P<i>\d+)(?:_)(?P<j>\d+)(?:\.png)").expect("Regex is cursed."));

/// Given a folder and a range of zoom levels, recursively creates tiles for all zoom levels.
pub fn render_zoom_levels(folder: impl AsRef<Path> + Send + Sync, mapid: i32, range: Range<i8>, backfill: [u8; 4]) -> CacheResult<()> {
    let zoom_levels = range.rev();
    for zoom in zoom_levels {
        let path = path!(folder / format!("{mapid}/{zoom}"));
        fs::create_dir_all(&path).map_err(|e| CacheError::io(e, path))?;

        let new_tile_coordinates = get_future_filenames(&folder, mapid, zoom + 1)?.into_iter();

        let func = |(p, i, j)| {
            let img = make_tile(&folder, mapid, zoom, p, i, j, backfill)?;
            let filename = path!(folder / format!("{mapid}/{zoom}/{p}_{i}_{j}.png"));

            match img.save(&filename) {
                Ok(()) => {}
                Err(ImageError::IoError(e)) => return Err(CacheError::io(e, filename)),
                Err(other) => panic!("{other}"),
            };
            Ok(())
        };

        new_tile_coordinates.progress().par_bridge().try_for_each(func)?;
    }
    Ok(())
}

fn to_coordinates(text: OsString) -> (i32, i32, i32) {
    let caps = RE.captures(text.to_str().unwrap()).unwrap();
    let p = caps.name("p").unwrap().as_str().parse::<i32>().unwrap();
    let i = caps.name("i").unwrap().as_str().parse::<i32>().unwrap();
    let j = caps.name("j").unwrap().as_str().parse::<i32>().unwrap();
    (p, i, j)
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
            Err(other_error) => panic!("{other_error}"),
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

    let new_tiles = fs::read_dir(&dir)
        .map_err(|e| CacheError::io(e, dir.clone()))?
        .collect::<io::Result<Vec<_>>>()
        .map_err(|e| CacheError::io(e, dir))?
        .into_iter()
        .map(|entry| entry.file_name())
        .map(to_coordinates)
        .map(|(p, i, j)| (p, i >> 1, j >> 1))
        .collect();

    Ok(new_tiles)
}
