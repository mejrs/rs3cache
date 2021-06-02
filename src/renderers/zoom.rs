use std::{collections::HashSet, ffi::OsString, fs, io, lazy::SyncLazy, ops::Range, sync::Mutex};

use async_std::{fs::File, prelude::*, task};
use futures::future::join_all;
use image::{imageops, io::Reader as ImageReader, ImageBuffer, ImageFormat, Rgba, RgbaImage};
use itertools::{iproduct, izip};
use progress_bar::progress_bar::ProgressBar;
use regex::Regex;

use crate::utils::{error::CacheResult, par::ParApply};

static RE: SyncLazy<Regex> = SyncLazy::new(|| Regex::new(r"(?P<p>\d+)(?:_)(?P<i>\d+)(?:_)(?P<j>\d+)(?:\.png)").expect("Regex is cursed."));

/// Given a folder and a range of zoom levels, recursively creates tiles for all zoom levels.
pub fn render_zoom_levels(folder: &str, mapid: i32, range: Range<i8>, backfill: [u8; 4]) -> CacheResult<()> {
    let final_zoom = range.start;
    let zoom_levels = range.rev();
    for zoom_level in zoom_levels {
        fs::create_dir_all(format!("{}/{}/{}", folder, mapid, zoom_level))?;

        let new_tile_coordinates = get_future_filenames(folder, mapid, zoom_level)?.into_iter();

        let length = new_tile_coordinates.size_hint().1.unwrap();

        let mut progress_bar = ProgressBar::new(length);
        progress_bar.print_info(
            "Creating",
            &format!("tiles for zoom level {}", zoom_level),
            progress_bar::color::Color::LightGreen,
            progress_bar::color::Style::Bold,
        );
        progress_bar.set_action("Rendering..", progress_bar::color::Color::Cyan, progress_bar::color::Style::Bold);
        let prog = Mutex::new(progress_bar);

        let func = |(p, i, j)| {
            let img = make_tile(folder, mapid, zoom_level, p, i, j, backfill).unwrap();
            let filename = format!("{}/{}/{}/{}_{}_{}.png", folder, mapid, zoom_level, p, i, j);
            img.save(filename).unwrap();
        };

        new_tile_coordinates.par_apply(|coords| {
            {
                prog.lock().unwrap().inc();
            }
            func(coords);
        });

        if zoom_level == final_zoom {
            prog.lock().unwrap().print_final_info(
                "Completed",
                "tiles for all zoom levels",
                progress_bar::color::Color::LightGreen,
                progress_bar::color::Style::Bold,
            );
        }
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
    folder: &str,
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

async fn get_file(filename: String) -> io::Result<Vec<u8>> {
    let mut file = File::open(filename).await?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await?;
    Ok(contents)
}

async fn get_files(folder: &str, mapid: i32, target_zoom: i8, target_plane: usize, target_i: usize, target_j: usize) -> Vec<io::Result<Vec<u8>>> {
    let mut files = Vec::new();
    for (di, dj) in iproduct!(0..=1, 0..=1) {
        let filename = format!(
            "{}/{}/{}/{}_{}_{}.png",
            folder,
            mapid,
            target_zoom + 1,
            target_plane,
            (target_i << 1) + di,
            (target_j << 1) + dj
        );
        files.push(get_file(filename));
    }
    join_all(files).await
}

fn get_future_filenames(folder: &str, mapid: i32, zoom_level: i8) -> CacheResult<HashSet<(usize, usize, usize)>> {
    let dir = format!("{}/{}/{}", folder, mapid, zoom_level + 1);
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
