use std::collections::BTreeMap;

use image::{GenericImage, Rgba, RgbaImage};
use ndarray::{ArrayBase, Dim, ViewRepr};

use super::{
    mapcore::CONFIG,
    tileshape::{OverlayShape, UnderlayShape},
};
use crate::definitions::{mapsquares::GroupMapSquare, overlays::Overlay, tiles::Tile, underlays::Underlay};

/// Applies ground colouring to the base image.
pub fn put(
    plane: usize,
    img: &mut RgbaImage,
    squares: &GroupMapSquare,
    underlay_definitions: &BTreeMap<u32, Underlay>,
    overlay_definitions: &BTreeMap<u32, Overlay>,
) {
    if let Ok(columns) = squares.core().indexed_columns() {
        columns.for_each(|(column, (x, y))| {
            for p in plane..=3_usize {
                let condition: bool = unsafe {
                    (p == 0 && plane == 0)
                        || (p == plane && column.uget(1).settings.unwrap_or(0) & 0x2 == 0)
                        || (p == plane + 1 && (column.uget(1).settings.unwrap_or(0) & 0x2 != 0))
                        || (p >= plane && column.uget(0).settings.unwrap_or(0) & 0x2 != 0)
                        || (plane == 0 && column.uget(p).settings.unwrap_or(0) & 0x8 != 0)
                };

                if condition {
                    // Underlays
                    if let Some((red, green, blue)) = get_underlay_colour(column, underlay_definitions, squares, p, x as usize, y as usize) {
                        let fill = Rgba([red, green, blue, 255u8]);

                        for (a, b) in UnderlayShape::new(column[p].shape, CONFIG.tile_size) {
                            unsafe {
                                debug_assert!(
                                    (CONFIG.tile_size * x + a) < img.width() && (CONFIG.tile_size * (63u32 - y) + b) < img.height(),
                                    "Index out of range."
                                );
                                img.unsafe_put_pixel(CONFIG.tile_size * x + a, CONFIG.tile_size * (63u32 - y) + b, fill)
                            }
                        }
                    }

                    // Overlays
                    if let Some(id) = column[p].overlay_id {
                        let ov = &overlay_definitions[&(id.checked_sub(1).expect("Not 100% sure about this invariant.") as u32)];
                        for colour in &[ov.primary_colour, ov.secondary_colour] {
                            if Some((255, 0, 255)) != *colour {
                                if let Some((red, green, blue)) = *colour {
                                    let fill = Rgba([red, green, blue, 255]);
                                    if column[p].shape.unwrap_or(0) != 0 {
                                        //dbg!(column[p]);
                                    }
                                    for (a, b) in OverlayShape::new(column[p].shape.unwrap_or(0), CONFIG.tile_size) {
                                        unsafe {
                                            debug_assert!(
                                                (CONFIG.tile_size * x + a) < img.width() && (CONFIG.tile_size * (63u32 - y) + b) < img.height(),
                                                "Index out of range."
                                            );
                                            img.unsafe_put_pixel(CONFIG.tile_size * x + a, CONFIG.tile_size * (63u32 - y) + b, fill)
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // The osrs client gets the average colour of textures here.
                    //
                    // The map has tiles, whose overlay_id points to an overlay config whose texture property points to a texture whose
                    // id points to a texture that has a field that refers to an index in a palette.
                    //
                    // To simplify the implementation, we simply hardcode these values.
                    // They don't change much, so this should be OK.
                    #[cfg(feature = "osrs")]
                    if let Some(id) = column[p].overlay_id {
                        if let Some(texture_id) =
                            &overlay_definitions[&(id.checked_sub(1).expect("Not 100% sure about this invariant.") as u32)].texture
                        {
                            let (red, green, blue) = match texture_id {
                                1 => (87, 108, 157),
                                2 => (70, 67, 63),
                                3 => (74, 45, 23),
                                11 => (64, 60, 56),
                                15 => (91, 87, 98),
                                23 => (50, 43, 28),
                                25 => (44, 103, 84),
                                31 => (213, 120, 8),
                                35 => (127, 110, 70),
                                43 => (87, 87, 78),
                                46 => (83, 77, 74),
                                51 => (118, 80, 37),
                                unknown => unimplemented!("unimplemented texture id {}", unknown),
                            };
                            let fill = Rgba([red, green, blue, 255]);

                            for (a, b) in OverlayShape::new(column[p].shape.unwrap_or(0), CONFIG.tile_size) {
                                unsafe {
                                    debug_assert!(
                                        (CONFIG.tile_size * x + a) < img.width() && (CONFIG.tile_size * (63u32 - y) + b) < img.height(),
                                        "Index out of range."
                                    );

                                    img.unsafe_put_pixel(CONFIG.tile_size * x + a, CONFIG.tile_size * (63u32 - y) + b, fill)
                                }
                            }
                        }
                    }
                }
            }
        })
    };
}

/// Averages out the [`Underlay`] colours, with a range specified by [`INTERP`].
fn get_underlay_colour(
    column: ArrayBase<ViewRepr<&Tile>, Dim<[usize; 1]>>,
    underlay_definitions: &BTreeMap<u32, Underlay>,
    squares: &GroupMapSquare,
    plane: usize,
    x: usize,
    y: usize,
) -> Option<(u8, u8, u8)> {
    // only compute a colour average if the tile has a underlay
    column[plane].underlay_id.map(|_| {
        let tiles = squares.tiles_iter(plane, x, y, CONFIG.interp);

        let underlays = tiles.filter_map(|elem| elem.underlay_id);

        let colours = underlays.map(|id| {
            (
                1usize, /* weight, todo? */
                underlay_definitions[&(id.checked_sub(1).unwrap() as u32)].colour.unwrap(),
            )
        });

        let (weight, (reds, greens, blues)) = colours
            .map(|(w, (r, g, b))| (w, (r as usize * w, g as usize * w, b as usize * w)))
            .fold((0, (0, 0, 0)), |(acc_w, (acc_r, acc_g, acc_b)), (w, (r, g, b))| {
                (acc_w + w, (acc_r + r, acc_g + g, acc_b + b))
            });

        (
            (reds / weight).try_into().unwrap(),
            (greens / weight).try_into().unwrap(),
            (blues / weight).try_into().unwrap(),
        )
    })
}
