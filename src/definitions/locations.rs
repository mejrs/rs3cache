use std::{
    fs::{self, File},
    io::Write,
};

#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, PyObjectProtocol};
use serde::{Serialize, Serializer};

use crate::{
    cache::buf::Buffer,
    definitions::{
        mapsquares::MapSquareIterator,
        tiles::{Tile, TileArray},
    },
    utils::{error::CacheResult, par::ParApply},
};

/// Describes whether this location is on the contained plane.
#[derive(Copy, Clone, Debug)]
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
#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Clone, Debug, Serialize)]
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
    pub(crate) fn dump_water_locations(i: u8, j: u8, file: Vec<u8>) -> Vec<Self> {
        let blanks = TileArray::from_elem((4, 64, 64), Tile::default());
        Self::dump(i, j, &blanks, file)
    }

    /// Constructor for [`Location`].
    pub fn dump(i: u8, j: u8, tiles: &TileArray, file: Vec<u8>) -> Vec<Self> {
        let mut buffer = Buffer::new(file);
        let mut locations = Vec::new();

        let mut id: i32 = -1;
        loop {
            match buffer.read_smarts() as i32 {
                0 => break,
                id_increment => {
                    id += id_increment;

                    let mut location = 0;
                    loop {
                        match buffer.read_unsigned_smart() {
                            0 => break,
                            location_increment => {
                                location += location_increment - 1;

                                let plane = (location >> 12) as u8;
                                let x = (location >> 6 & 0x3F) as u8;
                                let y = (location & 0x3F) as u8;

                                let data = buffer.read_unsigned_byte();
                                let r#type = data >> 2 & 0x1F;
                                let rotation = data & 0x3;

                                // some objects have offsets; not using this data atm
                                if data >= 0x80 {
                                    let sub_data = buffer.read_unsigned_byte();
                                    if sub_data != 0 {
                                        if sub_data & 0x1 != 0 {
                                            buffer.read_unsigned_short();
                                            buffer.read_unsigned_short();
                                            buffer.read_unsigned_short();
                                            buffer.read_unsigned_short();
                                        }
                                        if sub_data & 0x2 != 0 {
                                            buffer.read_unsigned_short();
                                        }
                                        if sub_data & 0x4 != 0 {
                                            buffer.read_unsigned_short();
                                        }
                                        if sub_data & 0x8 != 0 {
                                            buffer.read_unsigned_short();
                                        }
                                        if sub_data & 0x10 != 0 {
                                            buffer.read_unsigned_short();
                                        } else {
                                            if sub_data & 0x20 != 0 {
                                                buffer.read_unsigned_short();
                                            }
                                            if sub_data & 0x40 != 0 {
                                                buffer.read_unsigned_short();
                                            }
                                            if sub_data & 0x80 != 0 {
                                                buffer.read_unsigned_short();
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
        debug_assert_eq!(buffer.remaining(), 0);

        locations
    }
}

/// Saves all occurences of every object id as a `json` file to the folder `out/data/rs3/locations`.
pub fn export() -> CacheResult<()> {
    fs::create_dir_all("out/data/rs3/locations")?;

    let last_id = {
        let squares = MapSquareIterator::new()?;
        squares
            .filter_map(|sq| sq.take_locations().ok())
            .filter(|locs| !locs.is_empty())
            .map(|locs| locs.last().expect("locations stopped existing").id)
            .max()?
    };

    let squares = MapSquareIterator::new()?;
    let mut locs: Vec<_> = squares
        .filter_map(|sq| sq.take_locations().ok())
        .map(|locs| locs.into_iter().peekable())
        .collect();

    (0..=last_id)
        .map(|id| {
            (
                id,
                locs.iter_mut()
                    .flat_map(|iterator| std::iter::repeat_with(move || iterator.next_if(|loc| loc.id == id)).take_while(|item| item.is_some()))
                    .flatten()
                    .collect::<Vec<Location>>(),
            )
        })
        .par_apply(|(id, id_locs)| {
            if !id_locs.is_empty() && id != 83 {
                let mut file = File::create(format!("out/data/rs3/locations/{}.json", id)).unwrap();
                let data = serde_json::to_string_pretty(&id_locs).unwrap();
                file.write_all(data.as_bytes()).unwrap();
            }
        });

    Ok(())
}

/// Saves all occurences of every object id as a `json` file to the folder `out/data/rs3/locations`.
pub fn _export() -> CacheResult<()> {
    fs::create_dir_all("out/data/rs3/locations")?;

    let last_id = {
        let squares = MapSquareIterator::new()?;
        squares
            .filter_map(|sq| sq.take_locations().ok())
            .filter(|locs| !locs.is_empty())
            .map(|locs| locs.last().expect("locations stopped existing").id)
            .max()?
    };

    let squares = MapSquareIterator::new()?;
    let mut locs: Vec<_> = squares
        .filter_map(|sq| sq.take_locations().ok())
        .map(|locs| locs.into_iter().peekable())
        .collect();

    (0..=last_id)
        .map(|id| {
            (
                id,
                locs.iter_mut()
                    .flat_map(|iterator| std::iter::repeat_with(move || iterator.next_if(|loc| loc.id == id)).take_while(|item| item.is_some()))
                    .flatten()
                    .collect::<Vec<Location>>(),
            )
        })
        .par_apply(|(id, id_locs)| {
            if !id_locs.is_empty() && id != 83 {
                let mut file = File::create(format!("out/data/rs3/locations/{}.json", id)).unwrap();
                let data = serde_json::to_string_pretty(&id_locs).unwrap();
                file.write_all(data.as_bytes()).unwrap();
            }
        });

    Ok(())
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Location {
    #[getter]
    fn plane(&self) -> PyResult<Watery> {
        Ok(self.plane)
    }
    #[getter]
    fn i(&self) -> PyResult<u8> {
        Ok(self.i)
    }
    #[getter]
    fn j(&self) -> PyResult<u8> {
        Ok(self.j)
    }
    #[getter]
    fn x(&self) -> PyResult<u8> {
        Ok(self.x)
    }
    #[getter]
    fn y(&self) -> PyResult<u8> {
        Ok(self.y)
    }
    #[getter]
    fn id(&self) -> PyResult<u32> {
        Ok(self.id)
    }
    #[getter]
    fn r#type(&self) -> PyResult<u8> {
        Ok(self.r#type)
    }
    #[getter]
    fn rotation(&self) -> PyResult<u8> {
        Ok(self.rotation)
    }
}

#[cfg(feature = "pyo3")]
#[pyproto]
impl PyObjectProtocol for Location {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Location({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Location({})", serde_json::to_string(self).unwrap()))
    }
}
