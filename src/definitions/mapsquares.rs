//! Data comprising the game surface.
//!
//! The game map is divided in (potentially) 128 by 256 [`MapSquare`]s,
//! though this seems to be limited to 100 by 200.
//! These coordinates are referred to as `i` and `j`.
//!
//! Each [`MapSquare`] itself is comprised of 64 by 64 [`Tile`]s.
//! These coordinates are referred to as `x` and `y`.
//! They have four elevations, referred to as `p` or `plane`.
#![allow(unused_imports)]
use core::ops::{Range, RangeInclusive};
use std::{
    collections::{hash_map, BTreeMap, HashMap},
    iter::Zip,
};

use itertools::{iproduct, Product};
use ndarray::{iter::LanesIter, s, Axis, Dim};

#[cfg(feature = "osrs")]
use crate::cache::xtea::Xtea;
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

/// Represents a section of the game map
#[derive(Debug)]
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
    #[cfg(feature = "rs3")]
    water_locations: Result<Vec<Location>, CacheError>,
}

/// Iterator over a columns of planes with their x, y coordinates
pub type ColumnIter<'c> = Zip<LanesIter<'c, Tile, Dim<[usize; 2]>>, Product<Range<u32>, Range<u32>>>;

impl MapSquare {
    #[cfg(all(test, feature = "rs3"))]
    pub fn new(i: u8, j: u8) -> CacheResult<MapSquare> {
        assert!(i < 0x7F, "Index out of range.");
        let archive_id = (i as u32) | (j as u32) << 7;
        let archive = CacheIndex::new(IndexType::MAPSV2)?.archive(archive_id)?;
        Ok(Self::from_archive(archive))
    }

    #[cfg(feature = "osrs")]
    fn new(index: &CacheIndex<index::Initial>, xtea: Option<Xtea>, land: u32, tiles: u32, env: u32, i: u8, j: u8) -> CacheResult<MapSquare> {
        let land = index.archive_with_xtea(land, xtea).and_then(|mut arch| arch.take_file(&0));
        let tiles = index.archive(tiles).unwrap().take_file(&0).unwrap();
        let env = index.archive(env).unwrap();

        let tiles = Tile::dump(tiles);
        let locations = if let Ok(ok_land) = land {
            Ok(Location::dump(i, j, &tiles, ok_land))
        } else {
            Err(CacheError::ArchiveNotFoundError(5, 0))
        };

        Ok(MapSquare {
            i,
            j,
            tiles: Ok(tiles),
            locations,
        })
    }

    #[cfg(feature = "rs3")]
    pub(crate) fn from_archive(mut archive: Archive) -> MapSquare {
        let i = (archive.archive_id() & 0x7F) as u8;
        let j = (archive.archive_id() >> 7) as u8;
        let tiles = archive.take_file(&MapFileType::TILES).map(Tile::dump);
        let locations = match tiles {
            Ok(ref t) => archive.take_file(&MapFileType::LOCATIONS).map(|file| Location::dump(i, j, t, file)),
            // can't generally clone or copy error
            Err(CacheError::FileNotFoundError(i, a, f)) => Err(CacheError::FileNotFoundError(i, a, f)),
            _ => unreachable!(),
        };
        let water_locations = archive
            .take_file(&MapFileType::WATER_LOCATIONS)
            .map(|file| Location::dump_water_locations(i, j, file));

        MapSquare {
            i,
            j,
            tiles,
            locations,
            water_locations,
        }
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
    #[cfg(feature = "rs3")]
    pub fn get_water_locations(&self) -> Result<&Vec<Location>, &CacheError> {
        self.water_locations.as_ref()
    }

    /// Take its locations, consuming `self`.
    #[cfg(feature = "rs3")]
    pub fn take_water_locations(self) -> Result<Vec<Location>, CacheError> {
        self.water_locations
    }
}

/// Iterates over all [`MapSquare`]s in arbitrary order.
#[cfg(feature = "rs3")]
pub struct MapSquareIterator {
    inner: index::IntoIter,
}

#[cfg(feature = "rs3")]
impl MapSquareIterator {
    /// Constructor for MapSquareIterator.
    pub fn new() -> CacheResult<MapSquareIterator> {
        let inner = CacheIndex::new(IndexType::MAPSV2)?.into_iter();
        Ok(MapSquareIterator { inner })
    }
}

#[cfg(feature = "rs3")]
impl Iterator for MapSquareIterator {
    type Item = MapSquare;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(MapSquare::from_archive)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Iterates over all [`MapSquare`]s in arbitrary order.
#[cfg(feature = "osrs")]
pub struct MapSquareIterator {
    inner: CacheIndex<index::Initial>,
    mapping: BTreeMap<(&'static str, u8, u8), u32>,
    state: std::vec::IntoIter<(u8, u8)>,
}

#[cfg(feature = "osrs")]
impl MapSquareIterator {
    /// Constructor for MapSquareIterator.
    pub fn new() -> CacheResult<Self> {
        let inner = CacheIndex::new(IndexType::MAPSV2)?;

        let land_hashes: HashMap<i32, (u8, u8)> = iproduct!(0..100, 0..200)
            .map(|(i, j)| (crate::cache::hash::hash_djb2(format!("l{}_{}", i, j)), (i, j)))
            .collect();
        let map_hashes: HashMap<i32, (u8, u8)> = iproduct!(0..100, 0..200)
            .map(|(i, j)| (crate::cache::hash::hash_djb2(format!("m{}_{}", i, j)), (i, j)))
            .collect();
        let env_hashes: HashMap<i32, (u8, u8)> = iproduct!(0..100, 0..200)
            .map(|(i, j)| (crate::cache::hash::hash_djb2(format!("e{}_{}", i, j)), (i, j)))
            .collect();

        let mapping: BTreeMap<(&'static str, u8, u8), u32> = inner
            .metadatas()
            .iter()
            .map(|(_, m)| {
                let name_hash = m.name().unwrap();

                if let Some((i, j)) = land_hashes.get(&name_hash) {
                    (("l", *i, *j), m.archive_id())
                } else if let Some((i, j)) = map_hashes.get(&name_hash) {
                    (("m", *i, *j), m.archive_id())
                } else if let Some((i, j)) = env_hashes.get(&name_hash) {
                    (("e", *i, *j), m.archive_id())
                } else {
                    unreachable!()
                }
            })
            .collect();

        let state = mapping
            .keys()
            .filter_map(|(ty, i, j)| if *ty == "m" { Some((*i, *j)) } else { None })
            .collect::<Vec<_>>()
            .into_iter();

        Ok(MapSquareIterator { inner, mapping, state })
    }
}

#[cfg(feature = "osrs")]
impl Iterator for MapSquareIterator {
    type Item = MapSquare;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.next().map(|(i, j)| {
            let land = self.mapping.get(&("l", i, j)).unwrap();
            let map = self.mapping.get(&("m", i, j)).unwrap();
            let env = self.mapping.get(&("e", i, j)).unwrap();
            let xtea = self.inner.xteas().as_ref().unwrap().get(&(((i as u32) << 8) | j as u32));

            MapSquare::new(&self.inner, xtea.copied(), *land, *map, *env, i, j).unwrap()
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.state.size_hint()
    }
}

/// Iterates over [`GroupMapSquare`] in arbitrary order.
#[cfg(feature = "rs3")]
pub struct GroupMapSquareIterator {
    inner: index::IntoIterGrouped,
}

#[cfg(feature = "rs3")]
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

#[cfg(feature = "rs3")]
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

/// Iterates over [`GroupMapSquare`] in arbitrary order.
#[cfg(feature = "osrs")]
pub struct GroupMapSquareIterator {
    inner: CacheIndex<index::Initial>,
    range_i: RangeInclusive<i32>,
    range_j: RangeInclusive<i32>,
    mapping: BTreeMap<(u8, u8), (u32, u32, u32)>,
    state: std::vec::IntoIter<(u8, u8)>,
}

#[cfg(feature = "osrs")]
impl GroupMapSquareIterator {
    /// Constructor for [`GroupMapSquareIterator`].
    pub fn new(range_i: RangeInclusive<i32>, range_j: RangeInclusive<i32>) -> CacheResult<GroupMapSquareIterator> {
        let inner = CacheIndex::new(IndexType::MAPSV2)?;
        let mut mapping: BTreeMap<(u8, u8), (u32, u32, u32)> = BTreeMap::new();

        for (i, j) in iproduct!(0..100, 0..200) {
            let land = format!("l{}_{}", i, j);
            let tiles = format!("m{}_{}", i, j);
            let environment = format!("e{}_{}", i, j);

            let land_hash = crate::cache::hash::hash_djb2(land);
            let tiles_hash = crate::cache::hash::hash_djb2(tiles);
            let environment_hash = crate::cache::hash::hash_djb2(environment);

            let mut land = None;
            let mut tiles = None;
            let mut env = None;

            for (_, m) in inner.metadatas().iter() {
                if m.name() == Some(land_hash) {
                    land = Some(m.archive_id())
                }

                if m.name() == Some(tiles_hash) {
                    tiles = Some(m.archive_id())
                }

                if m.name() == Some(environment_hash) {
                    env = Some(m.archive_id())
                }
            }

            match (land, tiles, env) {
                (None, None, None) => {}
                (Some(l), Some(t), Some(e)) => {
                    mapping.insert((i, j), (l, t, e));
                }
                v => unimplemented!("{:?}", v),
            };
        }
        let state = mapping.keys().copied().collect::<Vec<_>>().into_iter();
        Ok(GroupMapSquareIterator {
            inner,
            range_i,
            range_j,
            mapping,
            state,
        })
    }
}

#[cfg(feature = "osrs")]
impl Iterator for GroupMapSquareIterator {
    type Item = GroupMapSquare;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.next().map(|(core_i, core_j)| {
            let coordinates =
                iproduct!(self.range_i.clone(), self.range_j.clone()).map(|(di, dj)| ((di + (core_i as i32)) as u8, ((dj + (core_j as i32)) as u8)));

            let mapsquares: HashMap<(u8, u8), MapSquare> = coordinates
                .filter_map(|(i, j)| {
                    if let Some((land, tiles, env)) = self.mapping.get(&(i, j)) {
                        let xtea = self.inner.xteas().as_ref().unwrap().get(&(((i as u32) << 8) | j as u32));
                        MapSquare::new(&self.inner, xtea.copied(), *land, *tiles, *env, i, j).ok()
                    } else {
                        None
                    }
                })
                .map(|sq| ((sq.i, sq.j), sq))
                .collect();
            assert!(mapsquares.contains_key(&(core_i, core_j)));

            GroupMapSquare { core_i, core_j, mapsquares }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.state.size_hint()
    }
}

impl ExactSizeIterator for GroupMapSquareIterator {}

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
#[cfg(feature = "osrs")]
impl MapFileType {
    /// Deserializes to the sequence of [`Location`]s in `self`.
    pub const LOCATIONS: &'static str = "l{}_{}";
    /// Deserializes to the [`TileArray`] of `self`.
    pub const TILES: &'static str = "m{}_{}";
    pub const ENVIRONMENT: &'static str = "e{}_{}";
}

#[allow(missing_docs)]
#[cfg(feature = "rs3")]
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

#[cfg(all(test, feature = "rs3"))]
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
