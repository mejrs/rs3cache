use std::collections::BTreeMap;

use image::{GenericImage, Rgba, RgbaImage};
use ndarray::{ArrayBase, Dim, ViewRepr};

use super::{tileshape, CONFIG};
#[cfg(feature = "legacy")]
use crate::definitions::flo::Flo;
use crate::definitions::{mapsquares::GroupMapSquare, tiles::Tile};
#[cfg(any(feature = "rs3", feature = "osrs"))]
use crate::definitions::{overlays::Overlay, underlays::Underlay};

/// Applies ground colouring to the base image.
pub fn put(
    plane: usize,
    img: &mut RgbaImage,
    squares: &GroupMapSquare,
    #[cfg(any(feature = "rs3", feature = "osrs"))] underlay_definitions: &BTreeMap<u32, Underlay>,
    #[cfg(any(feature = "rs3", feature = "osrs"))] overlay_definitions: &BTreeMap<u32, Overlay>,
    #[cfg(feature = "legacy")] flos: &BTreeMap<u32, Flo>,
) {
    if let Some(core) = squares.core() {
        if let Some(columns) = core.indexed_columns() {
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
                        #[cfg(any(feature = "rs3", feature = "osrs"))]
                        if let Some([red, green, blue]) = get_underlay_colour(column, underlay_definitions, squares, p, x as usize, y as usize) {
                            let fill = Rgba([red, green, blue, 255u8]);

                            tileshape::draw_underlay(column[p].shape, CONFIG.tile_size, |(a, b)| unsafe {
                                debug_assert!(
                                    (CONFIG.tile_size * x + a) < img.width() && (CONFIG.tile_size * (63u32 - y) + b) < img.height(),
                                    "Index out of range."
                                );
                                img.unsafe_put_pixel(CONFIG.tile_size * x + a, CONFIG.tile_size * (63u32 - y) + b, fill)
                            })
                        }

                        // Overlays
                        #[cfg(any(feature = "rs3", feature = "osrs"))]
                        if let Some(id) = column[p].overlay_id {
                            let ov = &overlay_definitions[&(id.checked_sub(1).expect("Not 100% sure about this invariant.") as u32)];
                            for colour in [ov.primary_colour, ov.secondary_colour] {
                                if Some([255, 0, 255]) != colour {
                                    if let Some([red, green, blue]) = colour {
                                        let fill = if id == 112 && colour == Some([255, 255, 255]) {
                                            // Gross hack to make ocean colours work past hd update
                                            Rgba([96, 118, 154, 255])
                                        } else {
                                            Rgba([red, green, blue, 255])
                                        };

                                        tileshape::draw_overlay(column[p].shape.unwrap_or(0), CONFIG.tile_size, |(a, b)| unsafe {
                                            debug_assert!(
                                                (CONFIG.tile_size * x + a) < img.width() && (CONFIG.tile_size * (63u32 - y) + b) < img.height(),
                                                "Index out of range."
                                            );
                                            img.unsafe_put_pixel(CONFIG.tile_size * x + a, CONFIG.tile_size * (63u32 - y) + b, fill)
                                        })
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
                                    91 => (171, 176, 181),
                                    119 => (50, 48, 44),
                                    120 => (6, 4, 4),
                                    121 | 122 => (87, 70, 62),
                                    125 | 126 => (50, 44, 36),
                                    unknown => unimplemented!("unimplemented texture id {}", unknown), //for the new water with sailing, probably (78, 95, 129)
                                };
                                let fill = Rgba([red, green, blue, 255]);

                                tileshape::draw_overlay(column[p].shape.unwrap_or(0), CONFIG.tile_size, |(a, b)| unsafe {
                                    debug_assert!(
                                        (CONFIG.tile_size * x + a) < img.width() && (CONFIG.tile_size * (63u32 - y) + b) < img.height(),
                                        "Index out of range."
                                    );

                                    img.unsafe_put_pixel(CONFIG.tile_size * x + a, CONFIG.tile_size * (63u32 - y) + b, fill)
                                });
                            }
                        }

                        // Underlays
                        #[cfg(feature = "legacy")]
                        if let Some([red, green, blue]) = get_underlay_colour(column, flos, squares, p, x as usize, y as usize) {
                            let fill = Rgba([red, green, blue, 255u8]);

                            tileshape::draw_underlay(column[p].shape, CONFIG.tile_size, |(a, b)| unsafe {
                                debug_assert!(
                                    (CONFIG.tile_size * x + a) < img.width() && (CONFIG.tile_size * (63u32 - y) + b) < img.height(),
                                    "Index out of range."
                                );
                                img.unsafe_put_pixel(CONFIG.tile_size * x + a, CONFIG.tile_size * (63u32 - y) + b, fill)
                            })
                        }

                        // Overlays
                        #[cfg(feature = "legacy")]
                        if let Some(id) = column[p].overlay_id {
                            let ov = &flos[&(id.checked_sub(1).expect("Not 100% sure about this invariant.") as u32)];
                            for colour in [ov.primary_colour, ov.secondary_colour] {
                                if Some([255, 0, 255]) != colour {
                                    if let Some([red, green, blue]) = colour {
                                        let fill = Rgba([red, green, blue, 255]);

                                        tileshape::draw_overlay(column[p].shape.unwrap_or(0), CONFIG.tile_size, |(a, b)| unsafe {
                                            debug_assert!(
                                                (CONFIG.tile_size * x + a) < img.width() && (CONFIG.tile_size * (63u32 - y) + b) < img.height(),
                                                "Index out of range."
                                            );
                                            img.unsafe_put_pixel(CONFIG.tile_size * x + a, CONFIG.tile_size * (63u32 - y) + b, fill)
                                        })
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
                        #[cfg(feature = "legacy")]
                        if let Some(id) = column[p].overlay_id {
                            if let Some(texture_id) = &flos[&(id.checked_sub(1).expect("Not 100% sure about this invariant.") as u32)].texture {
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
                                    91 => (171, 176, 181),
                                    119 => (50, 48, 44),
                                    unknown => unimplemented!("unimplemented texture id {}", unknown),
                                };
                                let fill = Rgba([red, green, blue, 255]);

                                tileshape::draw_overlay(column[p].shape.unwrap_or(0), CONFIG.tile_size, |(a, b)| unsafe {
                                    debug_assert!(
                                        (CONFIG.tile_size * x + a) < img.width() && (CONFIG.tile_size * (63u32 - y) + b) < img.height(),
                                        "Index out of range."
                                    );

                                    img.unsafe_put_pixel(CONFIG.tile_size * x + a, CONFIG.tile_size * (63u32 - y) + b, fill)
                                });
                            }
                        }
                    }
                }
            })
        };
    }
}

/// Averages out the [`Underlay`] colours, with a range specified by [`INTERP`].
#[cfg(any(feature = "rs3", feature = "osrs"))]
fn get_underlay_colour(
    column: ArrayBase<ViewRepr<&Tile>, Dim<[usize; 1]>>,
    underlay_definitions: &BTreeMap<u32, Underlay>,
    squares: &GroupMapSquare,
    plane: usize,
    x: usize,
    y: usize,
) -> Option<[u8; 3]> {
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
            .map(|(w, [r, g, b])| (w, (r as usize * w, g as usize * w, b as usize * w)))
            .fold((0, (0, 0, 0)), |(acc_w, (acc_r, acc_g, acc_b)), (w, (r, g, b))| {
                (acc_w + w, (acc_r + r, acc_g + g, acc_b + b))
            });

        [
            (reds / weight).try_into().unwrap(),
            (greens / weight).try_into().unwrap(),
            (blues / weight).try_into().unwrap(),
        ]
    })
}

/// Averages out the [`Underlay`] colours, with a range specified by [`INTERP`].
#[cfg(feature = "legacy")]
fn get_underlay_colour(
    column: ArrayBase<ViewRepr<&Tile>, Dim<[usize; 1]>>,
    flos: &BTreeMap<u32, Flo>,
    squares: &GroupMapSquare,
    plane: usize,
    x: usize,
    y: usize,
) -> Option<[u8; 3]> {
    // only compute a colour average if the tile has a underlay
    column[plane].underlay_id.map(|_| {
        let tiles = squares.tiles_iter(plane, x, y, CONFIG.interp);

        let underlays = tiles.filter_map(|elem| elem.underlay_id);

        let colours = underlays.map(|id| {
            (
                1usize, /* weight, todo? */
                flos[&(id.checked_sub(1).unwrap() as u32)].primary_colour.unwrap(),
            )
        });

        let (_weight, (reds, greens, blues)) = colours
            .map(|(w, [r, g, b])| (w, (r as usize * w, g as usize * w, b as usize * w)))
            .fold((0, (0, 0, 0)), |(acc_w, (acc_r, acc_g, acc_b)), (w, (r, g, b))| {
                (acc_w + w, (acc_r + r, acc_g + g, acc_b + b))
            });
        [
            (reds / 121).try_into().unwrap(),
            (greens / 121).try_into().unwrap(),
            (blues / 121).try_into().unwrap(),
        ]
    })
}
