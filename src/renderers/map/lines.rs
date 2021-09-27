use std::collections::BTreeMap;

use image::{GenericImage, Rgba, RgbaImage};

use crate::{
    definitions::{location_configs::LocationConfig, mapsquares::GroupMapSquare},
    renderers::map::{lineshape, mapcore::CONFIG},
    utils::color::Color,
};

/// Applies lines of doors, fences, walls and so on to the base image.
pub fn put(plane: usize, img: &mut RgbaImage, squares: &GroupMapSquare, location_config: &BTreeMap<u32, LocationConfig>) {
    if let Ok(locations) = squares.core().get_locations() {
        let tiles = squares.core().get_tiles().expect("always some if it has locations");
        locations
            .iter()
            .map(|loc| (loc, &(location_config[&(loc.id)])))
            .filter(|(location, properties)| unsafe {
                (location.r#type == 0 || location.r#type == 2 || location.r#type == 9)
                    && properties.mapscene.is_none()
                    && ((location.plane.matches(&0) && plane == 0)
                        || location.plane.contains(&(plane as u8))
                        || (tiles.uget((0, location.x as usize, location.y as usize)).settings.unwrap_or(0) & 0x2 != 0
                            && location.plane.inner() >= plane as u8)
                        || (tiles
                            .uget((location.plane.inner() as usize, location.x as usize, location.y as usize))
                            .settings
                            .unwrap_or(0)
                            & 0x8
                            != 0
                            && plane == 0))
            })
            .for_each(|(location, properties)| {
                let fill = if properties.unknown_19.contains(&1) || properties.actions.is_some() {
                    Rgba(Color::PURE_RED)
                } else {
                    Rgba(Color::WHITE)
                };

                lineshape::draw(location.r#type, location.rotation, CONFIG.tile_size, |(a, b)| unsafe {
                    debug_assert!(
                        (CONFIG.tile_size * location.x as u32 + a) < img.width()
                            && (CONFIG.tile_size * (63u32 - location.y as u32) + b) < img.height(),
                        "Index out of range."
                    );

                    img.unsafe_put_pixel(
                        CONFIG.tile_size * location.x as u32 + a,
                        CONFIG.tile_size * (63u32 - location.y as u32) + b,
                        fill,
                    );
                });
            });
    }
}
