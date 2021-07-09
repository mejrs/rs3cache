use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use image::{imageops, GenericImageView, ImageBuffer, Pixel, Rgba, RgbaImage};
use itertools::iproduct;
use progress_bar::progress_bar::ProgressBar;
use static_assertions::const_assert;

#[cfg(feature = "rs3")]
use crate::definitions::mapscenes::MapScene;
use crate::{
    definitions::{
        location_configs::LocationConfig,
        mapsquares::{GroupMapSquare, GroupMapSquareIterator},
        overlays::Overlay,
        sprites::{self, Sprite},
        underlays::Underlay,
    },
    renderers::{map::*, zoom},
    utils::{color::Color, error::CacheResult, par::ParApply},
};

/// Where to export the base tiles.

/// -1 is the "real" world map.
const MAPID: i32 = -1;

/// Scale factor.
///
/// Cannot be zero.
pub const SCALE: u32 = 4;
const_assert!(SCALE != 0);

/// The height and width of a [`Tile`](crate::definitions::tiles::Tile) in pixels.
pub const TILESIZE: u32 = 4 * SCALE;

/// The range at which underlays are blended.
pub const INTERP: isize = 5;
const_assert!(INTERP >= 0);

/// The height and width of a full [`MapSquare`](crate::definitions::mapsquares::MapSquare) in pixels.
pub const DIM: u32 = TILESIZE * 64;

/// The highest zoom level.
pub const INIT_ZOOM: i8 = 2;

const_assert!(INIT_ZOOM == 4 || INIT_ZOOM == 3 || INIT_ZOOM == 2);

/// Entry point for the map renderer.
pub fn render(path: impl AsRef<Path>) -> CacheResult<()> {
    let path = path.as_ref().to_str().unwrap();

    for zoom in 2..=4 {
        let folder = format!("{}/{}/{}", path, MAPID, zoom);
        fs::create_dir_all(folder)?;
    }

    let iter = GroupMapSquareIterator::new(-1_i32..=1_i32, -1_i32..=1_i32)?;

    inner_render(path, iter)?;

    zoom::render_zoom_levels(path, MAPID, -4..2, Color::ALPHA)?;
    Ok(())
}

// Separated for use in tests.

fn inner_render(path: &str, iter: GroupMapSquareIterator) -> CacheResult<()> {
    let location_definitions = LocationConfig::dump_all()?;
    let overlay_definitions = Overlay::dump_all()?;
    let underlay_definitions = Underlay::dump_all()?;

    #[cfg(feature = "rs3")]
    let mapscenes = MapScene::dump_all()?;
    #[cfg(feature = "rs3")]
    let sprite_ids = mapscenes.values().filter_map(|mapscene| mapscene.sprite_id).collect::<Vec<_>>();
    #[cfg(feature = "rs3")]
    let sprites = sprites::dumps(SCALE, sprite_ids)?;

    #[cfg(feature = "osrs")]
    let sprites = HashMap::new();

    #[cfg(feature = "rs3")]
    let prog = {
        let length = iter.size_hint().1.unwrap();
        let mut progress_bar = ProgressBar::new(length);
        progress_bar.print_info(
            "Creating",
            "map tiles",
            progress_bar::color::Color::LightGreen,
            progress_bar::color::Style::Bold,
        );
        progress_bar.set_action("Rendering..", progress_bar::color::Color::Cyan, progress_bar::color::Style::Bold);
        let prog = Arc::new(Mutex::new(progress_bar));
        prog
    };

    iter.par_apply(|gsq| {
        #[cfg(feature = "rs3")]
        {
            prog.lock().unwrap().inc();
        }

        render_tile(
            path,
            gsq,
            &location_definitions,
            &overlay_definitions,
            &underlay_definitions,
            #[cfg(feature = "rs3")]
            &mapscenes,
            &sprites,
        );
    });
    Ok(())
}

/// Responsible for rendering a single [`MapSquare`](crate::definitions::mapsquares::MapSquare).
pub fn render_tile(
    path: &str,
    squares: GroupMapSquare,
    location_config: &HashMap<u32, LocationConfig>,
    overlay_definitions: &HashMap<u32, Overlay>,
    underlay_definitions: &HashMap<u32, Underlay>,
    #[cfg(feature = "rs3")] mapscenes: &HashMap<u32, MapScene>,
    sprites: &HashMap<(u32, u32), Sprite>,
) {

    let func = |plane| {
        let mut img = RgbaImage::from_pixel(DIM, DIM, Rgba(Color::ALPHA));

        base::put(plane, &mut img, &squares, underlay_definitions, overlay_definitions);
        //lines::put(plane, &mut img, &squares, location_config);
        //mapscenes::put(plane, &mut img, &squares, location_config, mapscenes, sprites);
        img
    };

    let imgs = [func(0), func(1), func(2), func(3)];

    #[cfg(test)]
    {
        let filename = format!("tests/tiles/{}_{}_{}.png", 0, squares.core_i(), squares.core_j());
        imgs[0].save(filename).unwrap();
    }

    save_smallest(path, squares.core_i(), squares.core_j(), imgs);
}

type Img = ImageBuffer<Rgba<u8>, Vec<u8>>;

fn save_smallest(path: &str, i: u8, j: u8, imgs: [Img; 4]) {
    #![allow(unused_variables)]

    // SAFETY (2) these checks assure that...
    assert_eq!(DIM % 4, 0);
    for img in &imgs {
        assert_eq!(img.dimensions(), (DIM, DIM));
    }

    for plane in 0..=3 {
        let base = RgbaImage::from_fn(DIM, DIM, |x, y| {
            let mut i = (0..=plane).rev();

            loop {
                // SAFETY (1): this will always be valid....
                let p = unsafe { i.next().unwrap_unchecked() };

                // SAFETY (2):..these getters are always valid.
                let pixel = unsafe { imgs.get_unchecked(p).unsafe_get_pixel(x, y) };

                // SAFETY (1): ...as this exit condition always exits the loop if p == 0.
                if p == 0 || pixel[3] != 0 {
                    break if p == plane {
                        pixel
                    } else {
                        pixel.map_without_alpha(|channel| channel / 2)
                    };
                }
            }
        });

        if INIT_ZOOM >= 4 {
            let base_i = i as u32 * 4;
            let base_j = j as u32 * 4;
            for (x, y) in iproduct!(0..4u32, 0..4u32) {
                let sub_image = base.view((DIM / 4) * x, DIM - (DIM / 4) * (y + 1), DIM / 4, DIM / 4);
                debug_assert_eq!(sub_image.width(), 256);
                debug_assert_eq!(sub_image.height(), 256);

                #[cfg(not(test))]
                if sub_image.pixels().any(|(_, _, pixel)| pixel[3] != 0)
                /* don't save useless tiles */
                {
                    let filename = format!("{}/{}/{}/{}_{}_{}.png", path, MAPID, 4, plane, base_i + x, base_j + y);

                    sub_image.to_image().save(filename).unwrap();
                }
            }
        }

        if INIT_ZOOM >= 3 {
            let base_i = i as u32 * 2;
            let base_j = j as u32 * 2;
            for (x, y) in iproduct!(0..2u32, 0..2u32) {
                let sub_image = base.view((DIM / 2) * x, DIM - (DIM / 2) * (y + 1), DIM / 2, DIM / 2);

                #[cfg(not(test))]
                if sub_image.pixels().any(|(_, _, pixel)| pixel[3] != 0)
                /* don't save useless tiles */
                {
                    let resized = imageops::resize(&sub_image, 256, 256, imageops::FilterType::CatmullRom);

                    debug_assert_eq!(resized.width(), 256);
                    debug_assert_eq!(resized.height(), 256);
                    let filename = format!("{}/{}/{}/{}_{}_{}.png", path, MAPID, 3, plane, base_i + x, base_j + y);
                    resized.save(filename).unwrap();
                }
            }
        }

        if INIT_ZOOM >= 2 {
            let base_i = i as u32;
            let base_j = j as u32;

            let resized = imageops::resize(&base, 256, 256, imageops::FilterType::CatmullRom);

            debug_assert_eq!(resized.width(), 256);
            debug_assert_eq!(resized.height(), 256);

            #[cfg(not(test))]
            if resized.pixels().any(|&pixel| pixel[3] != 0)
            /* don't save useless tiles */
            {
                let filename = format!("{}/{}/{}/{}_{}_{}.png", path, MAPID, 2, plane, base_i, base_j);
                resized.save(filename).unwrap();
            }
        }
    }
}

#[doc(hidden)]
#[cfg(feature = "rs3")]
pub fn render_bench() -> CacheResult<()> {
    let path = "tests/tiles";
    fs::create_dir_all(path)?;
    let coordinates: Vec<(u8, u8)> = iproduct!(45..55, 45..55).collect();

    let iter = GroupMapSquareIterator::new_only(-1_i32..=1_i32, -1_i32..=1_i32, coordinates)?;
    inner_render(path, iter)?;

    Ok(())
}

#[doc(hidden)]
#[cfg(feature = "osrs")]
pub fn render_bench() -> CacheResult<()> {
    let path = "tests/tiles";
    fs::create_dir_all(path)?;
    let coordinates: Vec<(u8, u8)> = iproduct!(45..55, 45..55).collect();

    todo!();

    Ok(())
}

#[cfg(test)]
mod map_tests {
    use super::*;

    #[test]
    #[ignore]
    fn render_some() -> CacheResult<()> {
        let path = "tests/tiles";
        fs::create_dir_all(path)?;
        let coordinates: Vec<(u8, u8)> = vec![(50, 50), (41, 63), (47, 50), (56, 49), (34, 66), (33, 72), (49, 108), (43, 46)];

        let iter = GroupMapSquareIterator::new_only(-1_i32..=1_i32, -1_i32..=1_i32, coordinates)?;
        inner_render(path, iter)
    }
}
