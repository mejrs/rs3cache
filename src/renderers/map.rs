/// Renders the ground colours.
pub mod base;
/// Responsible for drawing lines - doors, fences, walls and so on.
pub mod lines;
/// Describes the shape of lines drawn by the map renderer.
pub mod lineshape;
/// Responsible for drawing [`MapScene`](crate::definitions::mapscenes::MapScene).
pub mod mapscenes;
/// Describes the shape of overlays drawn by the map renderer.
pub mod tileshape;

use std::{collections::BTreeMap, fs};

use ::error::Context;
use image::{GenericImageView, ImageBuffer, Pixel, Rgba, RgbaImage};
use indicatif::ProgressIterator;
use itertools::iproduct;
use path_macro::path;
use rayon::iter::{ParallelBridge, ParallelIterator};
use rs3cache_backend::error::{self, CacheResult};
use rs3cache_utils::color::Color;

#[cfg(feature = "legacy")]
use crate::definitions::flo::Flo;
#[cfg(any(feature = "rs3", feature = "2009_1_shim"))]
use crate::definitions::mapscenes::MapScene;
#[cfg(any(feature = "rs3", feature = "osrs"))]
use crate::definitions::{overlays::Overlay, underlays::Underlay};
use crate::{
    cli::Config,
    definitions::{
        location_configs::LocationConfig,
        mapsquares::{GroupMapSquare, GroupMapSquareIterator},
        sprites::{self, Sprite},
    },
    renderers::{scale, zoom},
};

pub struct RenderConfig {
    /// -1 is the "real" world map.
    pub map_id: i32,
    /// Scale factor, this cannot be zero.
    pub scale: u32,
    /// The height and width of a [`Tile`](crate::definitions::tiles::Tile) in pixels.
    pub tile_size: u32,
    /// The highest zoom level.
    pub initial_zoom: i8,
    /// The range at which underlays are blended.
    pub interp: isize,
    /// The height and width of a full [`MapSquare`](crate::definitions::mapsquares::MapSquare) in pixels.
    pub dim: u32,
}

impl RenderConfig {
    pub const fn fast() -> Self {
        Self {
            map_id: -1,
            scale: 4,
            tile_size: 16,
            interp: 5,
            dim: 1024,
            initial_zoom: 3,
        }
    }

    pub const fn detailed() -> Self {
        Self {
            map_id: -1,
            scale: 4,
            tile_size: 16,
            interp: 5,
            dim: 1024,
            initial_zoom: 4,
        }
    }
}

#[cfg(feature = "fast")]
pub static CONFIG: RenderConfig = RenderConfig::fast();

#[cfg(not(feature = "fast"))]
pub static CONFIG: RenderConfig = RenderConfig::detailed();

/// Entry point for the map renderer.
pub fn render(config: &Config) -> CacheResult<()> {
    const NAME: &str = "mapsquares";

    let map_id = CONFIG.map_id;
    for zoom in 2..=4 {
        let path = path!(config.output / NAME / format!("{map_id}/{zoom}"));

        fs::create_dir_all(&path).context(error::Io { path })?;
    }

    let iter = GroupMapSquareIterator::new(-1_i32..=1_i32, -1_i32..=1_i32, config)?;

    inner_render(config, NAME, iter)?;

    zoom::render_zoom_levels(config, NAME, map_id, -4..2, Color::ALPHA)?;
    Ok(())
}

// Separated for use in tests.

fn inner_render(config: &Config, name: &str, iter: GroupMapSquareIterator) -> CacheResult<()> {
    let location_definitions = LocationConfig::dump_all(config)?;

    #[cfg(any(feature = "rs3", feature = "osrs"))]
    let overlay_definitions = Overlay::dump_all(config)?;
    #[cfg(any(feature = "rs3", feature = "osrs"))]
    let underlay_definitions = Underlay::dump_all(config)?;

    #[cfg(any(feature = "rs3", feature = "2009_1_shim"))]
    let mapscenes = MapScene::dump_all(config)?;

    #[cfg(any(feature = "rs3", feature = "2009_1_shim"))]
    let sprites = sprites::dumps(
        CONFIG.scale,
        mapscenes.values().filter_map(|mapscene| mapscene.sprite_id).collect::<Vec<_>>(),
        config,
    )?;

    #[cfg(all(feature = "osrs", not(feature = "2009_1_shim")))]
    let sprites = sprites::dumps(CONFIG.scale, vec![317], config)?; // 317 is the sprite named "mapscene"

    #[cfg(feature = "legacy")]
    let sprites: BTreeMap<(u32, u32), Sprite> = sprites::get_mapscenes(CONFIG.scale, config)?;

    #[cfg(feature = "legacy")]
    let flos = Flo::dump_all(config)?;

    iter.progress().par_bridge().for_each(|gsq| {
        render_tile(
            config,
            name,
            gsq,
            &location_definitions,
            #[cfg(any(feature = "rs3", feature = "osrs"))]
            &overlay_definitions,
            #[cfg(any(feature = "rs3", feature = "osrs"))]
            &underlay_definitions,
            #[cfg(any(feature = "rs3", feature = "2009_1_shim"))]
            &mapscenes,
            #[cfg(feature = "legacy")]
            &flos,
            &sprites,
        );
    });
    Ok(())
}

/// Responsible for rendering a single [`MapSquare`](crate::definitions::mapsquares::MapSquare).
pub fn render_tile(
    config: &Config,
    name: &str,
    squares: GroupMapSquare,
    location_config: &BTreeMap<u32, LocationConfig>,
    #[cfg(any(feature = "rs3", feature = "osrs"))] overlay_definitions: &BTreeMap<u32, Overlay>,
    #[cfg(any(feature = "rs3", feature = "osrs"))] underlay_definitions: &BTreeMap<u32, Underlay>,
    #[cfg(any(feature = "rs3", feature = "2009_1_shim"))] mapscenes: &BTreeMap<u32, MapScene>,
    #[cfg(feature = "legacy")] flos: &BTreeMap<u32, Flo>,
    sprites: &BTreeMap<(u32, u32), Sprite>,
) {
    let func = |plane| {
        let backfill = Rgba(Color::ALPHA);

        let mut img = RgbaImage::from_pixel(CONFIG.dim, CONFIG.dim, backfill);

        base::put(
            plane,
            &mut img,
            &squares,
            #[cfg(any(feature = "rs3", feature = "osrs"))]
            underlay_definitions,
            #[cfg(any(feature = "rs3", feature = "osrs"))]
            overlay_definitions,
            #[cfg(feature = "legacy")]
            flos,
        );
        lines::put(plane, &mut img, &squares, location_config);
        mapscenes::put(
            plane,
            &mut img,
            &squares,
            location_config,
            #[cfg(any(feature = "rs3", feature = "2009_1_shim"))]
            mapscenes,
            sprites,
        );
        img
    };

    let imgs = [func(0), func(1), func(2), func(3)];

    #[cfg(test)]
    {
        let filename = format!("test_data/tiles/{}_{}_{}.png", 0, squares.core_i(), squares.core_j());
        imgs[0].save(filename).unwrap();
    }

    save_smallest(config, name, squares.core_i(), squares.core_j(), imgs);
}

type Img = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub fn save_smallest(config: &Config, name: &str, i: u8, j: u8, imgs: [Img; 4]) {
    let map_id = CONFIG.map_id;

    // SAFETY (2) these checks assure that...
    assert_eq!(CONFIG.dim % 4, 0);
    for img in &imgs {
        assert_eq!(img.dimensions(), (CONFIG.dim, CONFIG.dim));
    }

    for plane in 0..=3 {
        let base = RgbaImage::from_fn(CONFIG.dim, CONFIG.dim, |x, y| {
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

        if CONFIG.initial_zoom >= 4 {
            let base_i = i as u32 * 4;
            let base_j = j as u32 * 4;
            for (x, y) in iproduct!(0..4u32, 0..4u32) {
                let sub_image = base.view(
                    (CONFIG.dim / 4) * x,
                    CONFIG.dim - (CONFIG.dim / 4) * (y + 1),
                    CONFIG.dim / 4,
                    CONFIG.dim / 4,
                );
                debug_assert_eq!(sub_image.width(), 256);
                debug_assert_eq!(sub_image.height(), 256);

                if cfg!(not(test)) && sub_image.pixels().any(|(_, _, pixel)| pixel[3] != 0)
                /* don't save useless tiles */
                {
                    let xx = base_i + x;
                    let yy = base_j + y;
                    let filename = path!(config.output / name / format!("{map_id}/4/{plane}_{xx}_{yy}.png"));
                    sub_image.to_image().save(filename).unwrap();
                }
            }
        }

        if CONFIG.initial_zoom >= 3 {
            let base_i = i as u32 * 2;
            let base_j = j as u32 * 2;
            for (x, y) in iproduct!(0..2u32, 0..2u32) {
                let sub_image = base.view(
                    (CONFIG.dim / 2) * x,
                    CONFIG.dim - (CONFIG.dim / 2) * (y + 1),
                    CONFIG.dim / 2,
                    CONFIG.dim / 2,
                );

                if cfg!(not(test)) && sub_image.pixels().any(|(_, _, pixel)| pixel[3] != 0)
                /* don't save useless tiles */
                {
                    let resized = scale::resize_half(*sub_image);

                    debug_assert_eq!(resized.width(), 256);
                    debug_assert_eq!(resized.height(), 256);
                    let xx = base_i + x;
                    let yy = base_j + y;
                    let filename = path!(config.output / name / format!("{map_id}/3/{plane}_{xx}_{yy}.png"));
                    resized.save(filename).unwrap();
                }
            }
        }

        if CONFIG.initial_zoom >= 2 {
            let base_i = i as u32;
            let base_j = j as u32;

            let resized = scale::resize_quarter(base);

            debug_assert_eq!(resized.width(), 256);
            debug_assert_eq!(resized.height(), 256);

            if cfg!(not(test)) && resized.pixels().any(|&pixel| pixel[3] != 0)
            /* don't save useless tiles */
            {
                let filename = path!(config.output / name / format!("{map_id}/2/{plane}_{base_i}_{base_j}.png"));
                resized.save(filename).unwrap();
            }
        }
    }
}

#[doc(hidden)]
#[cfg(feature = "rs3")]
pub fn render_bench() -> CacheResult<()> {
    let config = crate::cli::Config::default();
    let path = "test_data/tiles";
    fs::create_dir_all(path).unwrap();
    let coordinates: Vec<(u8, u8)> = iproduct!(45..55, 45..55).collect();

    let iter = GroupMapSquareIterator::new_only(-1_i32..=1_i32, -1_i32..=1_i32, coordinates, &config)?;
    inner_render(&config, path, iter)?;

    Ok(())
}

#[cfg(all(test, feature = "rs3"))]
mod map_tests {
    use super::*;

    #[test]
    #[ignore]
    fn render_some() -> CacheResult<()> {
        let config = Config::default();

        let path = "test_data/tiles";
        fs::create_dir_all(path).unwrap();
        let coordinates: Vec<(u8, u8)> = vec![(50, 50), (41, 63), (47, 50), (56, 49), (34, 66), (33, 72), (49, 108), (43, 46)];

        let iter = GroupMapSquareIterator::new_only(-1_i32..=1_i32, -1_i32..=1_i32, coordinates, &config)?;
        inner_render(&config, path, iter)
    }
}
