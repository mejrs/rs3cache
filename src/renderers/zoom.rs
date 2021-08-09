use std::{collections::HashSet, ffi::OsString, fs, io, lazy::SyncLazy, ops::Range, path::Path};

use async_std::{fs::File, prelude::*, task};
use fstrings::{f, format_args_f};
use futures::future::join_all;
use image::{imageops, io::Reader as ImageReader, ImageBuffer, ImageFormat, Rgba, RgbaImage};
use indicatif::ProgressIterator;
use itertools::{iproduct, izip};
use path_macro::path;
use regex::Regex;

use crate::utils::{error::CacheResult, par::ParApply};

static RE: SyncLazy<Regex> = SyncLazy::new(|| Regex::new(r"(?P<p>\d+)(?:_)(?P<i>\d+)(?:_)(?P<j>\d+)(?:\.png)").expect("Regex is cursed."));

/// Given a folder and a range of zoom levels, recursively creates tiles for all zoom levels.
pub fn render_zoom_levels(folder: impl AsRef<Path> + Send + Sync, mapid: i32, range: Range<i8>, backfill: [u8; 4]) -> CacheResult<()> {
    let zoom_levels = range.rev();
    for zoom in zoom_levels {
        fs::create_dir_all(path!(folder / f!("{mapid}/{zoom}")))?;

        let new_tile_coordinates = get_future_filenames(&folder, mapid, zoom + 1)?.into_iter();

        let func = |(p, i, j)| {
            let img = make_tile(&folder, mapid, zoom, p, i, j, backfill).unwrap();
            let filename = path!(folder / f!("{mapid}/{zoom}/{p}_{i}_{j}.png"));
            img.save(filename).unwrap();
        };

        new_tile_coordinates.progress().par_apply(func);
    }
    Ok(())
}

fn to_coordinates(text: OsString) -> CacheResult<(usize, usize, usize)> {
    let caps = RE.captures(text.to_str().unwrap()).unwrap();
    let p = caps.name("p").unwrap().as_str().parse::<usize>()?;
    let i = caps.name("i").unwrap().as_str().parse::<usize>()?;
    let j = caps.name("j").unwrap().as_str().parse::<usize>()?;
    Ok((p, i, j))
}

fn make_tile(
    folder: impl AsRef<Path>,
    mapid: i32,
    target_zoom: i8,
    target_plane: usize,
    target_i: usize,
    target_j: usize,
    backfill: [u8; 4],
) -> CacheResult<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut base = RgbaImage::from_fn(512, 512, |_, _| Rgba(backfill));

    let files = task::block_on(get_files(folder, mapid, target_zoom, target_plane, target_i, target_j));

    for ((di, dj), file) in izip!(iproduct!(0..=1, 0..=1), files) {
        match file {
            Ok(f) => {
                let img = ImageReader::with_format(io::Cursor::new(f), ImageFormat::Png).decode()?;
                imageops::overlay(&mut base, &img, (256 * di) as u32, 256 * (1 - dj) as u32);
            }
            // can be missing; if so, swallow
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(other_error) => return Err(other_error.into()),
        }
    }
    let scaled = imageops::resize(&base, 256, 256, imageops::FilterType::CatmullRom);
    Ok(scaled)
}

async fn get_file(filename: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut file = File::open(filename.as_ref()).await?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await?;
    Ok(contents)
}

async fn get_files(
    folder: impl AsRef<Path>,
    mapid: i32,
    target_zoom: i8,
    target_plane: usize,
    target_i: usize,
    target_j: usize,
) -> Vec<io::Result<Vec<u8>>> {
    let mut files = Vec::new();
    for (di, dj) in iproduct!(0..=1, 0..=1) {
        let i = (target_i << 1) + di;
        let j = (target_j << 1) + dj;
        let zoom = target_zoom + 1;
        let filename = path!(folder / f!("{mapid}/{zoom}/{target_plane}_{i}_{j}.png"));
        files.push(get_file(filename));
    }
    join_all(files).await
}

fn get_future_filenames(folder: impl AsRef<Path>, mapid: i32, zoom: i8) -> CacheResult<HashSet<(usize, usize, usize)>> {
    let dir = path!(folder / f!("{mapid}/{zoom}"));

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
