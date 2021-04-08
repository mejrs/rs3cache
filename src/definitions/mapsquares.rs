//! Data comprising the game surface.
//!
//! The game map is divided in (potentially) 128 by 256 [`MapSquare`]s,
//! though this seems to be limited to 100 by 200.
//! These coordinates are referred to as `i` and `j`.
//!
//! Each [`MapSquare`] itself is comprised of 64 by 64 [`Tile`]s.
//! These coordinates are referred to as `x` and `y`.
//! They have four elevations, referred to as `p` or `plane`.

use crate::{
    cache::{
        arc::Archive,
        index::{self, CacheIndex},
        indextype::IndexType,
    },
    definitions::{
        locations::Location,
        tiles::{Tile, TileArray},
    },
    utils::{
        error::{CacheError, CacheResult},
        rangeclamp::RangeClamp,
    },
};
use core::ops::{Range, RangeInclusive};
use itertools::{iproduct, Product};
use ndarray::{iter::LanesIter, s, Axis, Dim};
use std::{
    collections::{hash_map, HashMap},
    iter::Zip,
};

/// Represents a section of the game map
pub struct MapSquare {
    /// The horizontal [`MapSquare`] coordinate.
    ///
    /// It can have any value in the range `0..=100`.
    pub i: u8,

    /// The vertical [`MapSquare`] coordinate.
    ///
    /// It can have any value in the range `0..=200`.
    pub j: u8,

    /// Data on the tiles it contains.
    tiles: Result<TileArray, CacheError>,

    /// All locations in this [`MapSquare`].
    ///
    /// Locations can overlap on surrounding mapsquares.
    locations: Result<Vec<Location>, CacheError>,

    /// All water locations in this [`MapSquare`].
    ///
    /// Locations can overlap on surrounding mapsquares.
    water_locations: Result<Vec<Location>, CacheError>,
}

/// Iterator over a columns of planes with their x, y coordinates
pub type ColumnIter<'c> = Zip<LanesIter<'c, Tile, Dim<[usize; 2]>>, Product<Range<u32>, Range<u32>>>;

impl MapSquare {
    #[cfg(test)]
    pub fn new(i: u8, j: u8) -> CacheResult<MapSquare> {
        assert!(i < 0x7F, "Index out of range.");
        let archive_id = (i as u32) | (j as u32) << 7;
        let archive = CacheIndex::new(IndexType::MAPSV2)?.archive(archive_id)?;
        Ok(Self::from_archive(archive))
    }

    pub(crate) fn from_archive(mut archive: Archive) -> MapSquare {
        let i = (archive.archive_id() & 0x7F) as u8;
        let j = (archive.archive_id() >> 7) as u8;
        let tiles = archive.take_file(&MapFileType::TILES).map(Tile::dump);
        let locations = match tiles {
            Ok(ref t) => archive.take_file(&MapFileType::LOCATIONS).map(|file| Location::dump(i, j, &t, file)),
            // can't generally clone or copy error
            Err(CacheError::FileNotFoundError(i, a, f)) => Err(CacheError::FileNotFoundError(i, a, f)),
            _ => unreachable!(),
        };
        let water_locations = archive.take_file(&MapFileType::WATER_LOCATIONS).map(|file| Location::dump_water_locations(i, j, file));

        MapSquare { i, j, tiles, locations, water_locations }
    }
    /// Iterator over a columns of planes with their x, y coordinates
    pub fn indexed_columns(&self) -> Result<ColumnIter, &CacheError> {
        self.get_tiles()
            .map(|tiles| tiles.lanes(Axis(0)).into_iter().zip(iproduct!(0..64u32, 0..64u32)))
    }

    /// Returns a view over the `tiles` field, if present
    pub fn get_tiles(&self) -> Result<&TileArray, &CacheError> {
        self.tiles.as_ref()
    }

    /// Returns a view over the `locations` field, if present.
    pub fn get_locations(&self) -> Result<&Vec<Location>, &CacheError> {
        self.locations.as_ref()
    }

    /// Take its locations, consuming `self`.
    pub fn take_locations(self) -> Result<Vec<Location>, CacheError> {
        self.locations
    }

     /// Returns a view over the `locations` field, if present.
     pub fn get_water_locations(&self) -> Result<&Vec<Location>, &CacheError> {
        self.water_locations.as_ref()
    }

    /// Take its locations, consuming `self`.
    pub fn take_water_locations(self) -> Result<Vec<Location>, CacheError> {
        self.water_locations
    }
}

/// Iterates over all [`MapSquare`]s in arbitrary order.
pub struct MapSquareIterator {
    inner: index::IntoIter,
}

impl MapSquareIterator {
    /// Constructor for MapSquareIterator.
    pub fn new() -> CacheResult<MapSquareIterator> {
        let inner = CacheIndex::new(IndexType::MAPSV2)?.into_iter();
        Ok(MapSquareIterator { inner })
    }
}

impl Iterator for MapSquareIterator {
    type Item = MapSquare;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(MapSquare::from_archive)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Iterates over [`GroupMapSquare`] in arbitrary order.
pub struct GroupMapSquareIterator {
    inner: index::IntoIterGrouped,
}

impl GroupMapSquareIterator {
    /// Constructor for [`GroupMapSquareIterator`].
    pub fn new(dx: RangeInclusive<i32>, dy: RangeInclusive<i32>) -> CacheResult<GroupMapSquareIterator> {
        let inner = CacheIndex::new(IndexType::MAPSV2)?.grouped(dx, dy).into_iter();
        Ok(GroupMapSquareIterator { inner })
    }

    /// Constructor for [`GroupMapSquareIterator`], but limited to the [`MapSquare`]s in `coordinates`.
    pub fn new_only(dx: RangeInclusive<i32>, dy: RangeInclusive<i32>, coordinates: Vec<(u8, u8)>) -> CacheResult<GroupMapSquareIterator> {
        let archive_ids = coordinates.into_iter().map(|(i, j)| (i as u32) | (j as u32) << 7).collect::<Vec<u32>>();
        let inner = CacheIndex::new(IndexType::MAPSV2)?.retain(archive_ids).grouped(dx, dy).into_iter();
        Ok(GroupMapSquareIterator { inner })
    }
}

impl Iterator for GroupMapSquareIterator {
    type Item = GroupMapSquare;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|group| {
            let (core_i, core_j) = ((group.core_id() & 0x7F) as u8, (group.core_id() >> 7) as u8);
            let mapsquares = group
                .into_iter()
                .map(MapSquare::from_archive)
                .map(|sq| ((sq.i, sq.j), sq))
                .collect::<HashMap<_, _>>();
            GroupMapSquare { core_i, core_j, mapsquares }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A group of adjacent [`MapSquare`]s.
///
/// Necessary for operations that need to care about surrounding mapsquares.
pub struct GroupMapSquare {
    core_i: u8,
    core_j: u8,
    mapsquares: HashMap<(u8, u8), MapSquare>,
}

impl GroupMapSquare {
    /// The horizontal coordinate of the central [`MapSquare`].
    ///
    /// It can have any value in the range `0..100`.
    #[inline(always)]
    pub fn core_i(&self) -> u8 {
        self.core_i
    }

    /// The vertical coordinate of the central [`MapSquare`].
    ///
    /// It can have any value in the range `0..200`.
    #[inline(always)]
    pub fn core_j(&self) -> u8 {
        self.core_j
    }

    /// Returns a reference to the central [`MapSquare`].
    pub fn core(&self) -> &MapSquare {
        &self.mapsquares[&(self.core_i, self.core_j)]
    }

    /// Iterates over all [`MapSquare`]s of `self` in arbitrary order.
    pub fn iter(&self) -> hash_map::Iter<'_, (u8, u8), MapSquare> {
        self.mapsquares.iter()
    }

    /// Returns a view over a specific [`MapSquare`]..
    pub fn get(&self, key: &(u8, u8)) -> Option<&MapSquare> {
        self.mapsquares.get(key)
    }

    /// Returns a view over all tiles within `interp` of the [`Tile`] at `plane, x, y`.
    pub fn tiles_iter(&self, plane: usize, x: usize, y: usize, interp: isize) -> Box<dyn Iterator<Item = &Tile> + '_> {
        let low_x = x as isize - interp;
        let upper_x = x as isize + interp + 1;
        let low_y = y as isize - interp;
        let upper_y = y as isize + interp + 1;

        Box::new(
            self.iter()
                .filter_map(move |((i, j), sq)| {
                    sq.get_tiles().ok().map(|tiles| {
                        let di = (*i as isize) - (self.core_i as isize);
                        let dj = (*j as isize) - (self.core_j as isize);
                        ((di, dj), tiles)
                    })
                })
                .flat_map(move |((di, dj), tiles)| {
                    tiles
                        .slice(s![
                            plane,
                            ((low_x - 64 * di)..(upper_x - 64 * di)).clamp(0, 64),
                            ((low_y - 64 * dj)..(upper_y - 64 * dj)).clamp(0, 64)
                        ])
                        .into_iter()
                }),
        )
    }

    /// Returns a view over all locations in all [`MapSquare`]s of `self` in arbitrary order.
    pub fn all_locations_iter(&self) -> Box<dyn Iterator<Item = &Location> + '_> {
        Box::new(
            self.iter()
                .filter_map(|(_k, square)| square.get_locations().ok())
                .flat_map(IntoIterator::into_iter),
        )
    }
}

/// Enumeration of the files in the [MAPSV2](IndexType::MAPSV2) archives.
pub struct MapFileType;

#[allow(missing_docs)]
impl MapFileType {
    /// Deserializes to the sequence of [`Location`]s in `self`.
    pub const LOCATIONS: u32 = 0;
    /// Deserializes to a sequence of underwater [`Location`]s in `self`. Not implemented.
    pub const WATER_LOCATIONS: u32 = 1;
    /// Deserializes to a sequence of all npcs in `self`.
    /// Only mapsquares which used to have a "zoom around" login screen,
    /// or are derived from one that had, have this file.
    pub const NPCS: u32 = 2;
    /// Deserializes to the [`TileArray`] of `self`.
    pub const TILES: u32 = 3;
    /// Deserializes to the underwater [`TileArray`] of `self`.
    pub const WATER_TILES: u32 = 4;
    pub const UNKNOWN_5: u32 = 5;
    pub const UNKNOWN_6: u32 = 6;
    pub const UNKNOWN_7: u32 = 7;
    pub const UNKNOWN_8: u32 = 8;
    pub const UNKNOWN_9: u32 = 9;
}

#[cfg(test)]
mod map_tests {
    use super::*;

    #[test]
    fn loc_0_50_50_9_16_is_trapdoor() -> CacheResult<()> {
        let id = 36687_u32;
        let square = MapSquare::new(50, 50)?;
        assert!(square
            .get_locations()?
            .iter()
            .any(|loc| loc.id == id && loc.plane.matches(&0) && loc.x == 9 && loc.y == 16));
        Ok(())
    }

    #[test]
    fn get_tile() -> CacheResult<()> {
        let square = MapSquare::new(49, 54)?;
        let _tile = square.get_tiles()?.get([0, 24, 25]);
        Ok(())
    }
}
