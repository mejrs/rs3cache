use ndarray::{Array, ArrayBase, Dim, OwnedRepr};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::cache::buf::Buffer;

/// Type alias for the 4x64x64 array of [`Tile`]s in a [`MapSquare`](crate::definitions::mapsquares::MapSquare).
pub type TileArray = ArrayBase<OwnedRepr<Tile>, Dim<[usize; 3]>>;

/// Describes the properties of a tile in a [`MapSquare`](crate::definitions::mapsquares::MapSquare).
#[cfg_eval]
#[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Default, Debug, Copy, Clone)]
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

#[cfg(feature = "pyo3")]
#[pymethods]
impl Tile {
    #[getter]
    fn shape(&self) -> PyResult<Option<u8>> {
        Ok(self.shape)
    }
    #[getter]
    fn overlay_id(&self) -> PyResult<Option<u16>> {
        Ok(self.overlay_id)
    }
    #[getter]
    fn settings(&self) -> PyResult<Option<u8>> {
        Ok(self.settings)
    }
    #[getter]
    fn underlay_id(&self) -> PyResult<Option<u16>> {
        Ok(self.underlay_id)
    }
    #[getter]
    fn height(&self) -> PyResult<Option<u8>> {
        Ok(self.height)
    }
}

impl Tile {
    /// Constructor for a sequence of [`Tile`]s.
    #[cfg(feature = "rs3")]
    pub fn dump(file: Vec<u8>) -> TileArray {
        let mut buffer = Buffer::new(file);

        let tiles = Array::from_shape_simple_fn((4, 64, 64), || {
            let mut tile = Tile::default();

            let [flag_1, flag_2, flag_3, flag_4, ..] = buffer.read_bitflags();

            if flag_1 {
                tile.shape = Some(buffer.read_unsigned_byte());
                tile.overlay_id = Some(buffer.read_unsigned_smart());
            }

            if flag_2 {
                tile.settings = Some(buffer.read_unsigned_byte());
            }

            if flag_3 {
                tile.underlay_id = Some(buffer.read_unsigned_smart());
            }

            if flag_4 {
                tile.height = Some(buffer.read_unsigned_byte());
            }

            tile
        });

        if buffer.remaining() != 0 {
            //println!("{}", buffer.remaining());
        }
        tiles
    }

    #[cfg(feature = "osrs")]
    pub fn dump(file: Vec<u8>) -> TileArray {
        let mut buffer = Buffer::new(file);

        let tiles = Array::from_shape_simple_fn((4, 64, 64), || {
            let mut tile = Tile::default();

            loop {
                match buffer.read_unsigned_byte() {
                    0 => break tile,
                    1 => {
                        tile.height = Some(buffer.read_unsigned_byte());
                        break tile;
                    }
                    opcode if opcode <= 49 => {
                        tile.shape = Some(opcode - 2);
                        tile.overlay_id = Some(buffer.read_unsigned_byte() as u16);
                    }
                    opcode if opcode <= 81 => tile.settings = Some(opcode - 49),
                    opcode => tile.underlay_id = Some((opcode - 81) as u16),
                }
            }
        });

        if buffer.remaining() != 0 {
            //println!("{}", buffer.remaining());
        }
        tiles
    }
}
