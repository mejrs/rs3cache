#[allow(unused_imports)]
use bytes::{Buf, Bytes};
use ndarray::{Array, ArrayBase, Dim, OwnedRepr};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[allow(unused_imports)]
use rs3cache_backend::buf::{BufExtra, ReadError};
use serde::Serialize;
/// Type alias for the 4x64x64 array of [`Tile`]s in a [`MapSquare`](crate::definitions::mapsquares::MapSquare).
pub type TileArray = ArrayBase<OwnedRepr<Tile>, Dim<[usize; 3]>>;

/// Describes the properties of a tile in a [`MapSquare`](crate::definitions::mapsquares::MapSquare).

#[cfg_attr(feature = "pyo3", pyclass(frozen, get_all))]
#[derive(Default, Debug, Copy, Clone, Serialize)]
pub struct Tile {
    /// Reference to a [shape](crate::renderers::map::tileshape).
    pub shape: Option<u8>,

    /// Reference to an [`Overlay`](crate::definitions::overlays::Overlay).
    pub overlay_id: Option<u16>,

    /// This tile's settings.
    pub settings: Option<u8>,

    /// Reference to an [`Underlay`](crate::definitions::underlays::Underlay).
    pub underlay_id: Option<u16>,

    /// The height of the tile.
    pub height: Option<u8>,
}

impl Tile {
    /// Constructor for a sequence of [`Tile`]s.
    #[cfg(any(feature = "rs3", feature = "2013_shim"))]
    pub fn dump(buffer: &mut Bytes) -> TileArray {
        Array::from_shape_simple_fn((4, 64, 64), || {
            let mut tile = Tile::default();

            let [flag_1, flag_2, flag_3, flag_4, ..] = buffer.get_bitflags();

            if flag_1 {
                tile.shape = Some(buffer.get_u8());
                tile.overlay_id = Some(buffer.get_unsigned_smart());
            }

            if flag_2 {
                tile.settings = Some(buffer.get_u8());
            }

            if flag_3 {
                tile.underlay_id = Some(buffer.get_unsigned_smart());
            }

            if flag_4 {
                tile.height = Some(buffer.get_u8());
            }

            tile
        })
    }

    #[cfg(feature = "legacy")]
    pub fn dump(buffer: &mut Bytes) -> TileArray {
        let shape = Self::try_dump(buffer.clone(), false).unwrap();

        Array::from_shape_vec((4, 64, 64), shape).unwrap()
    }

    #[cfg(all(feature = "osrs", not(feature = "2013_shim")))]
    pub fn dump(buffer: &mut Bytes) -> TileArray {
        // This is a hack to deal with the changing of the tile format
        //
        // Rather than introducing a new feature for it,
        // try to figure out the correct format at runtime
        let shape = match Self::try_dump(buffer.clone(), true) {
            Ok(shape) => shape,
            Err(_) => Self::try_dump(buffer.clone(), false).unwrap(),
        };

        Array::from_shape_vec((4, 64, 64), shape).unwrap()
    }

    #[cfg(any(feature = "osrs", feature = "legacy"))]
    fn try_dump(mut buffer: Bytes, use_post_oct_2022: bool) -> Result<Vec<Tile>, ReadError> {
        let producer = || try {
            let mut tile = Tile::default();

            loop {
                let opcode = if use_post_oct_2022 {
                    buffer.try_get_u16()?
                } else {
                    buffer.try_get_u8()? as u16
                };

                match opcode {
                    0 => break tile,
                    1 => {
                        tile.height = Some(buffer.try_get_u8()?);
                        break tile;
                    }
                    opcode @ 2..=49 => {
                        tile.shape = Some(opcode as u8 - 2);

                        let id = if use_post_oct_2022 {
                            buffer.try_get_u16()?
                        } else {
                            buffer.try_get_u8()? as u16
                        };
                        tile.overlay_id = Some(id);
                    }
                    opcode @ 50..=81 => tile.settings = Some(opcode as u8 - 49),
                    opcode @ 82.. => tile.underlay_id = Some(opcode - 81),
                }
            }
        };

        let ret = std::iter::repeat_with(producer).take(4 * 64 * 64).collect();
        if buffer.is_empty() {
            ret
        } else {
            Err(ReadError::not_exhausted())
        }
    }
}
