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
                                    1 => (104, 125, 169),   // 6847913
                                    2 => (70, 70, 70),      // 4605510
                                    3 => (75, 43, 25),      // 4926233
                                    11 => (62, 62, 62),     // 4079166
                                    15 => (102, 102, 102),  // 6710886
                                    23 => (48, 42, 29),     // 3156509
                                    25 => (50, 110, 87),    // 3305047
                                    31 => (240, 124, 21),   // 15760405
                                    35 => (125, 115, 68),   // 8221508
                                    43 => (89, 89, 89),     // 5855577
                                    46 => (84, 84, 84),     // 5526612
                                    51 => (126, 81, 42),    // 8278314
                                    91 => (204, 204, 204),  // 13421772
                                    119 => (48, 48, 48),    // 3158064
                                    120 => (48, 42, 29),    // 3156509
                                    121 => (105, 84, 74),   // 6902858
                                    122 => (105, 84, 74),   // 6902858
                                    125 => (54, 49, 41),    // 3551529
                                    126 => (54, 49, 41),    // 3551529
                                    130 => (91, 116, 170),  // 5993642
                                    131 => (87, 111, 162),  // 5730210
                                    132 => (83, 105, 152),  // 5466520
                                    133 => (74, 93, 137),   // 4873609
                                    134 => (71, 89, 132),   // 4675972
                                    135 => (102, 139, 175), // 6720431
                                    136 => (98, 136, 173),  // 6457517
                                    137 => (91, 131, 170),  // 5997482
                                    138 => (87, 125, 162),  // 5733794
                                    140 => (91, 139, 170),  // 5999530
                                    141 => (87, 132, 162),  // 5735586
                                    142 => (85, 128, 156),  // 5603484
                                    143 => (83, 125, 152),  // 5471640
                                    144 => (74, 110, 137),  // 4877961
                                    145 => (118, 119, 183), // 7763895
                                    146 => (102, 103, 175), // 6711215
                                    147 => (91, 94, 170),   // 5988010
                                    148 => (83, 84, 152),   // 5461144
                                    149 => (80, 81, 147),   // 5263763
                                    150 => (124, 118, 161), // 8156833
                                    151 => (115, 108, 155), // 7564443
                                    152 => (107, 100, 143), // 7038095
                                    153 => (103, 97, 138),  // 6775178
                                    154 => (97, 90, 129),   // 6380161
                                    155 => (113, 114, 158), // 7434910
                                    156 => (108, 110, 155), // 7106203
                                    157 => (103, 105, 148), // 6777236
                                    158 => (97, 98, 138),   // 6382218
                                    159 => (87, 88, 124),   // 5724284
                                    160 => (93, 143, 192),  // 6131648
                                    161 => (81, 135, 188),  // 5343164
                                    162 => (75, 131, 186),  // 4948922
                                    163 => (71, 122, 172),  // 4684460
                                    164 => (64, 110, 155),  // 4222619
                                    165 => (136, 151, 173), // 8951725
                                    166 => (126, 142, 167), // 8294055
                                    167 => (118, 135, 161), // 7767969
                                    168 => (108, 127, 155), // 7110555
                                    169 => (100, 117, 143), // 6583695
                                    170 => (76, 148, 177),  // 5018801
                                    171 => (68, 139, 177),  // 4492209
                                    172 => (65, 133, 170),  // 4294058
                                    173 => (61, 123, 160),  // 4029344
                                    174 => (54, 110, 147),  // 3567251
                                    175 => (116, 129, 129), // 7635329
                                    176 => (105, 119, 119), // 6911863
                                    180 => (102, 151, 175), // 6723503
                                    181 => (91, 146, 170),  // 6001322
                                    182 => (85, 135, 156),  // 5605276
                                    183 => (83, 129, 152),  // 5472664
                                    184 => (74, 116, 137),  // 4879497
                                    185 => (118, 135, 161), // 7767969
                                    186 => (108, 127, 155), // 7110555
                                    187 => (100, 117, 143), // 6583695
                                    188 => (93, 109, 134),  // 6122886
                                    189 => (87, 102, 124),  // 5727868
                                    208 => (32, 47, 61),    // 2109245
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
