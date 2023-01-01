use std::hash::Hash;

use bytes::{Buf, Bytes};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rs3cache_backend::buf::BufExtra;
use serde::{Serialize, Serializer};

use crate::definitions::tiles::TileArray;
/// Describes whether this location is on the contained plane.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Watery {
    /// It's on the contained plane
    True(u8),
    /// It's on (contained plane - 1).
    False(u8),
}

impl Watery {
    /// Returns the contained value directly.
    pub fn inner(&self) -> u8 {
        match self {
            Self::True(value) => *value,
            Self::False(value) => *value,
        }
    }

    /// Returns whether this location is actually on the given plane.
    pub fn matches(&self, plane: &u8) -> bool {
        match self {
            Self::True(value) => *value == *plane + 1,
            Self::False(value) => *value == *plane,
        }
    }

    /// Directly compare the contained value to a given plane.
    pub fn contains(&self, plane: &u8) -> bool {
        match self {
            Self::True(value) => *value == *plane,
            Self::False(value) => *value == *plane,
        }
    }
}

impl Serialize for Watery {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::True(val) => serializer.serialize_i8((*val as i8) - 1),
            Self::False(val) => serializer.serialize_i8(*val as i8),
        }
    }
}

#[cfg(feature = "pyo3")]
impl IntoPy<PyObject> for Watery {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            Self::True(val) => ((val as i8) - 1).into_py(py),
            Self::False(val) => (val as i8).into_py(py),
        }
    }
}

/// A location, also referred to as an "object".
#[cfg_eval]
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
#[cfg_attr(feature = "pyo3", pyclass(frozen))]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Location {
    /// The plane a.k.a elevation.
    ///
    /// It can have any value in the range `0..=3`.
    pub plane: Watery,
    /// The horizontal [`MapSquare`](crate::definitions::mapsquares::MapSquare) coordinate.
    ///
    /// It can have any value in the range `0..=100`.
    ///
    /// All operations on this value should use explicit wrapping arithmetic.
    pub i: u8,
    /// The vertical [`MapSquare`](crate::definitions::mapsquares::MapSquare) coordinate.
    ///
    /// It can have any value in the range `0..=200`.
    ///
    /// All operations on this value should use explicit wrapping arithmetic.
    pub j: u8,
    /// The horizontal coordinate inside its [`MapSquare`](crate::definitions::mapsquares::MapSquare).
    ///
    /// It can have any value in the range `0..=63`.
    /// Locations that are not 1x1 have their most western tile as this coordinate.
    pub x: u8,
    /// The vertical coordinate inside its [`MapSquare`](crate::definitions::mapsquares::MapSquare).
    ///
    /// It can have any value in the range `0..=63`.
    /// Locations that are not 1x1 have their most southern tile as this coordinate.
    pub y: u8,
    /// The id corresponding to its [`LocationConfig`](crate::definitions::location_configs::LocationConfig).
    pub id: u32,
    /// The type of this location. The [`models`](crate::definitions::location_config::LocationConfig.models) field of its
    /// [`LocationConfig`](crate::definitions::location_configs::LocationConfig) maps models to its type.
    // #[cfg_attr(feature ="pyo3", pyo3(get))]
    pub r#type: u8,
    /// Its rotation, also known as "orientation".
    pub rotation: u8,
}

impl Location {
    // todo: fix this with water tiles
    #[cfg(any(feature = "rs3", feature = "2013_4_shim"))]
    pub(crate) fn dump_water_locations(i: u8, j: u8, buffer: Bytes) -> Vec<Self> {
        let blanks = TileArray::default((4, 64, 64));
        Self::dump(i, j, &blanks, buffer)
    }

    /// Constructor for [`Location`].
    pub fn dump(i: u8, j: u8, tiles: &TileArray, mut buffer: Bytes) -> Vec<Self> {
        let mut locations = Vec::new();

        let mut id: i32 = -1;

        loop {
            match buffer.get_smarts() as i32 {
                0 => break locations,
                id_increment => {
                    id += id_increment;

                    let mut location = 0;
                    loop {
                        match buffer.get_unsigned_smart() {
                            0 => break,
                            location_increment => {
                                location += location_increment - 1;

                                let plane = (location >> 12) as u8;
                                let x = (location >> 6 & 0x3F) as u8;
                                let y = (location & 0x3F) as u8;

                                let data = buffer.get_u8();
                                let r#type = data >> 2 & 0x1F;
                                let rotation = data & 0x3;

                                // some objects have offsets; not using this data atm
                                #[cfg(feature = "rs3")]
                                if data >= 0x80 {
                                    let sub_data = buffer.get_u8();
                                    if sub_data != 0 {
                                        if sub_data & 0x1 != 0 {
                                            buffer.get_u16();
                                            buffer.get_u16();
                                            buffer.get_u16();
                                            buffer.get_u16();
                                        }
                                        if sub_data & 0x2 != 0 {
                                            buffer.get_u16();
                                        }
                                        if sub_data & 0x4 != 0 {
                                            buffer.get_u16();
                                        }
                                        if sub_data & 0x8 != 0 {
                                            buffer.get_u16();
                                        }
                                        if sub_data & 0x10 != 0 {
                                            buffer.get_u16();
                                        } else {
                                            if sub_data & 0x20 != 0 {
                                                buffer.get_u16();
                                            }
                                            if sub_data & 0x40 != 0 {
                                                buffer.get_u16();
                                            }
                                            if sub_data & 0x80 != 0 {
                                                buffer.get_u16();
                                            }
                                        }
                                    }
                                }
                                let watery_plane = if tiles[[1, x as usize, y as usize]].settings.unwrap_or(0) & 0x2 != 0 {
                                    Watery::True(plane)
                                } else {
                                    Watery::False(plane)
                                };
                                let loc = Location {
                                    plane: watery_plane,
                                    i,
                                    j,
                                    x,
                                    y,
                                    id: id as u32,
                                    r#type,
                                    rotation,
                                };
                                locations.push(loc);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Location {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Location({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Location({})", serde_json::to_string(self).unwrap()))
    }

    fn __hash__(&self) -> PyResult<u64> {
        use std::{collections::hash_map::DefaultHasher, hash::Hasher};

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        Ok(hasher.finish())
    }

    fn __richcmp__(&self, other: Location, op: pyo3::class::basic::CompareOp) -> PyResult<bool> {
        match op {
            pyo3::class::basic::CompareOp::Eq => Ok(*self == other),
            _ => todo!(),
        }
    }
}

#[cfg(feature = "pyo3")]
impl ToPyObject for Location {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        (*self).into_py(py)
    }
}
