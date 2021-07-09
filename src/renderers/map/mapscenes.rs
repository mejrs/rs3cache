use std::collections::HashMap;

use image::{GenericImage, GenericImageView, RgbaImage};
use itertools::iproduct;

use crate::{
    definitions::{location_configs::LocationConfig,  mapsquares::GroupMapSquare, sprites::Sprite},
    renderers::map::mapcore::TILESIZE,
    utils::rangeclamp::RangeClamp,
};

#[cfg(feature = "rs3")]
use crate::definitions::mapscenes::MapScene;

/// Applies [`MapScene`]s to the base image.
#[cfg(feature = "rs3")]
pub fn put(
    plane: usize,
    img: &mut RgbaImage,
    squares: &GroupMapSquare,
    location_config: &HashMap<u32, LocationConfig>,
    mapscenes: &HashMap<u32, MapScene>,
    sprites: &HashMap<(u32, u32), Sprite>,
) {
    squares
        .all_locations_iter()
        .filter_map(|loc| {
            if loc.plane.matches(&(plane as u8)) {
                location_config[&(loc.id)].mapscene.and_then(|mapscene_id| {
                    mapscenes[&(mapscene_id as u32)]
                        .sprite_id
                        // sprites is constructed with ids from
                        // mapscenes so it should always be in the map.
                        .map(|sprite_id| (loc, &sprites[&(sprite_id, 0)]))
                })
            } else {
                None
            }
        })
        .for_each(|(loc, sprite)| {
            let offset_a = TILESIZE as i32 * ((loc.i as i32 - squares.core_i() as i32) * 64 + loc.x as i32);
            let offset_b = TILESIZE as i32 * (63 - (loc.j as i32 - squares.core_j() as i32) * 64 - loc.y as i32);

            let dim_a = sprite.width() as i32;
            let dim_b = sprite.height() as i32;

            let range_a = (offset_a..(offset_a + dim_a)).clamp(0, img.width() as i32);
            let range_b = ((offset_b - dim_b / 2)..(offset_b + dim_b / 2)).clamp(0, img.height() as i32);

            for (a, b) in iproduct!(range_a, range_b) {
                let sprite_a = (a - offset_a) as u32;
                let sprite_b = (b - (offset_b - dim_b / 2)) as u32;

                let sprite_pixel = unsafe {
                    debug_assert!(sprite_a < sprite.width() && sprite_b < sprite.height(), "Index out of range.");
                    sprite.unsafe_get_pixel(sprite_a, sprite_b)
                };
                if sprite_pixel[3] != 0 {
                    unsafe {
                        debug_assert!((a as u32) < img.width() && (b as u32) < img.height(), "Index out of range.");
                        img.unsafe_put_pixel(a as u32, b as u32, sprite_pixel)
                    };
                }
            }
        });
}
