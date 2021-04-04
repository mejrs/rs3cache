use crate::cache::buf::Buffer;
use ndarray::{Array, ArrayBase, Dim, OwnedRepr};
use pyo3::prelude::*;

/// Type alias for the 4x64x64 array of [`Tile`]s in a [`MapSquare`](crate::definitions::mapsquares::MapSquare).
pub type TileArray = ArrayBase<OwnedRepr<Tile>, Dim<[usize; 3]>>;

/// Describes the properties of a tile in a [`MapSquare`](crate::definitions::mapsquares::MapSquare).
#[pyclass]
#[derive(Default, Debug, Copy, Clone)]
pub struct Tile {
    /// Reference to an [`OverlayShape`](crate::renderers::map::tileshape::OverlayShape).
    #[pyo3(get)]
    pub shape: Option<u8>,

    /// Reference to an [`Overlay`](crate::definitions::overlays::Overlay).
    #[pyo3(get)]
    pub overlay_id: Option<u16>,

    /// This tile's settings.
    #[pyo3(get)]
    pub settings: Option<u8>,

    /// Reference to an [`Underlay`](crate::definitions::underlays::Underlay).
    #[pyo3(get)]
    pub underlay_id: Option<u16>,

    /// The height of the tile.
    #[pyo3(get)]
    pub height: Option<u8>,
}

impl Tile {
    /// Constructor for a sequence of [`Tile`]s.
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
}
