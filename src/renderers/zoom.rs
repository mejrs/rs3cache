use std::{collections::HashSet, ffi::OsString, fs, io, ops::Range, path::Path, sync::LazyLock};

use ::error::Context;
use image::{imageops, ImageBuffer, ImageError, Rgba, RgbaImage};
use itertools::Itertools;
use path_macro::path;
use rayon::iter::ParallelIterator;
use regex::Regex;
use rs3cache_backend::error::{self, CacheResult};
use rs3cache_utils::bar::Render;

use crate::{cli::Config, renderers::scale};

static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?P<p>\d+)(?:_)(?P<i>\d+)(?:_)(?P<j>\d+)(?:\.png)").expect("Regex is cursed."));

/// Given a folder and a range of zoom levels, recursively creates tiles for all zoom levels.
pub fn render_zoom_levels(config: &Config, name: &str, mapid: i32, range: Range<i8>, backfill: [u8; 4]) -> CacheResult<()> {
    let zoom_levels = range.rev();
    for zoom in zoom_levels {
        let path = path!(config.output / name / format!("{mapid}/{zoom}"));
        fs::create_dir_all(&path).context(error::Io { path })?;

        let new_tile_coordinates = get_future_filenames(config, name, mapid, zoom + 1)?.into_iter();

        let func = |((p, i, j), _)| {
            let img = make_tile(&config.output, name, mapid, zoom, p, i, j, backfill)?;
            let path = path!(config.output / &name / format!("{mapid}/{zoom}/{p}_{i}_{j}.png"));

            match img.save(&path) {
                Ok(()) => {}
                Err(ImageError::IoError(e)) => return Err(e).context(error::Io { path }),
                Err(other) => panic!("{other}"),
            };
            Ok(())
        };

        new_tile_coordinates.render(format!("{name} zoom level {zoom}")).try_for_each(func)?;
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
    name: &str,
    mapid: i32,
    target_zoom: i8,
    target_plane: i32,
    target_i: i32,
    target_j: i32,
    backfill: [u8; 4],
) -> CacheResult<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut base = RgbaImage::from_fn(512, 512, |_, _| Rgba(backfill));

    let files = get_files(folder, name, mapid, target_zoom, target_plane, target_i, target_j);

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
    name: &str,
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
        let filename = path!(folder / name / format!("{mapid}/{zoom}/{target_plane}_{i}_{j}.png"));
        ((di, dj), image::open(filename))
    })
}

fn get_future_filenames(config: &Config, name: &str, mapid: i32, zoom: i8) -> CacheResult<HashSet<(i32, i32, i32)>> {
    let path = path!(config.output / name / format!("{mapid}/{zoom}"));

    fs::read_dir(&path)
        .with_context(|| error::Io { path: path.clone() })?
        .map_ok(|entry| {
            let name = entry.file_name();
            let (p, i, j) = to_coordinates(name);
            (p, i >> 1, j >> 1)
        })
        .collect::<Result<_, _>>()
        .context(error::Io { path })
}
