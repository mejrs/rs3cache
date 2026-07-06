use core::ops::RangeInclusive;
use std::collections::{BTreeMap, HashMap};

use ::error::Context;
use itertools::iproduct;
use rs3cache_backend::{
    error::{self, CacheResult},
    index::{self, CacheIndex},
};

use crate::definitions::{
    indextype::{IndexType, MapFileType},
    mapsquares::{GroupMapSquare, MapSquare, MapSquares, MAX_REGIONS},
};
impl MapSquares {
    pub fn new(config: &crate::cli::Config) -> CacheResult<MapSquares> {
        let index = CacheIndex::new(IndexType::MAPSV2, config.input.clone())?;
        let land_hashes: HashMap<i32, (u8, u8)> = iproduct!(0..100, 0..200)
            .map(|(i, j)| {
                (
                    rs3cache_backend::hash::hash_djb2(format!("{}{}_{}", MapFileType::LOCATIONS, i, j)),
                    (i, j),
                )
            })
            .collect();
        let map_hashes: HashMap<i32, (u8, u8)> = iproduct!(0..100, 0..200)
            .map(|(i, j)| (rs3cache_backend::hash::hash_djb2(format!("{}{}_{}", MapFileType::TILES, i, j)), (i, j)))
            .collect();
        let env_hashes: HashMap<i32, (u8, u8)> = iproduct!(0..100, 0..200)
            .map(|(i, j)| {
                (
                    rs3cache_backend::hash::hash_djb2(format!("{}{}_{}", MapFileType::ENVIRONMENT, i, j)),
                    (i, j),
                )
            })
            .collect();

        let mapping: BTreeMap<_, _> = index
            .metadatas()
            .iter()
            .flat_map(|(_, m)| {
                let name_hash = m.name()?;

                let ret = if let Some((i, j)) = land_hashes.get(&name_hash) {
                    (("l", *i, *j), m.archive_id())
                } else if let Some((i, j)) = map_hashes.get(&name_hash) {
                    (("m", *i, *j), m.archive_id())
                } else if let Some((i, j)) = env_hashes.get(&name_hash) {
                    (("e", *i, *j), m.archive_id())
                } else {
                    (("ul_or_um_dont_care", 0, 0), m.archive_id())
                };
                Some(ret)
            })
            .collect();

        Ok(MapSquares {
            index,
            mapping: if mapping.is_empty() { None } else { Some(mapping) },
        })
    }

    pub fn get(&self, i: u8, j: u8) -> CacheResult<MapSquare> {
        if let Some(mapping) = &self.mapping {
            let land = mapping
                .get(&("l", i, j))
                .with_context(|| index::ArchiveMissingNamed {
                    index_id: 5,
                    name: format!("l{i}_{j}"),
                })
                .context(error::Integrity)?;
            let map = mapping.get(&("m", i, j)).unwrap();
            let env = mapping.get(&("e", i, j)).copied();
            let xtea = self.index.xteas().as_ref().unwrap().get(&(((i as u32) << 8) | j as u32));

            MapSquare::new_named(&self.index, xtea.copied(), *land, *map, env, i, j)
        } else {
            MapSquare::new(&self.index, i, j)
        }
    }
}

/// Iterates over all [`MapSquare`]s in arbitrary order.
pub struct MapSquareIterator {
    pub(crate) mapsquares: MapSquares,
    pub(crate) state: std::vec::IntoIter<(u8, u8)>,
}

impl Iterator for MapSquareIterator {
    type Item = CacheResult<MapSquare>;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.next().map(|(i, j)| self.mapsquares.get(i, j))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.state.size_hint()
    }
}

impl ExactSizeIterator for MapSquareIterator {}

/// Iterates over [`GroupMapSquare`] in arbitrary order.
pub struct GroupMapSquareIterator {
    inner: CacheIndex<index::Initial>,
    range_i: RangeInclusive<i32>,
    range_j: RangeInclusive<i32>,
    mapping: Option<BTreeMap<(&'static str, u8, u8), u32>>,
    state: std::vec::IntoIter<(u8, u8)>,
}

impl GroupMapSquareIterator {
    /// Constructor for [`GroupMapSquareIterator`].
    pub fn new(range_i: RangeInclusive<i32>, range_j: RangeInclusive<i32>, config: &crate::cli::Config) -> CacheResult<GroupMapSquareIterator> {
        let inner = CacheIndex::new(IndexType::MAPSV2, config.input.clone())?;

        let land_hashes: HashMap<i32, (u8, u8)> = iproduct!(0..100, 0..200)
            .map(|(i, j)| (rs3cache_backend::hash::hash_djb2(format!("l{i}_{j}")), (i, j)))
            .collect();
        let map_hashes: HashMap<i32, (u8, u8)> = iproduct!(0..100, 0..200)
            .map(|(i, j)| (rs3cache_backend::hash::hash_djb2(format!("m{i}_{j}")), (i, j)))
            .collect();
        let env_hashes: HashMap<i32, (u8, u8)> = iproduct!(0..100, 0..200)
            .map(|(i, j)| (rs3cache_backend::hash::hash_djb2(format!("ul{i}_{j}")), (i, j)))
            .collect();

        let mapping: BTreeMap<(&'static str, u8, u8), u32> = inner
            .metadatas()
            .iter()
            .flat_map(|(_, m)| {
                let name_hash = m.name()?;

                let ret = if let Some((i, j)) = land_hashes.get(&name_hash) {
                    (("l", *i, *j), m.archive_id())
                } else if let Some((i, j)) = map_hashes.get(&name_hash) {
                    (("m", *i, *j), m.archive_id())
                } else if let Some((i, j)) = env_hashes.get(&name_hash) {
                    (("e", *i, *j), m.archive_id())
                } else {
                    (("ul_or_um_dont_care", 0, 0), m.archive_id())
                };
                Some(ret)
            })
            .collect();
        let mapping = if mapping.is_empty() { None } else { Some(mapping) };

        let state = if let Some(mapping) = &mapping {
            mapping
                .keys()
                .filter_map(|(ty, i, j)| if *ty == "m" { Some((*i, *j)) } else { None })
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            inner
                .metadatas()
                .keys()
                .flat_map(|id| {
                    if *id < MAX_REGIONS {
                        Some(((id >> 8) as u8, (id & 0xFF) as u8))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .into_iter()
        };

        Ok(GroupMapSquareIterator {
            inner,
            range_i,
            range_j,
            mapping,
            state,
        })
    }
}

impl Iterator for GroupMapSquareIterator {
    type Item = GroupMapSquare;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.next().map(|(core_i, core_j)| {
            let coordinates =
                iproduct!(self.range_i.clone(), self.range_j.clone()).map(|(di, dj)| ((di + (core_i as i32)) as u8, ((dj + (core_j as i32)) as u8)));

            let mapsquares: HashMap<(u8, u8), MapSquare> = coordinates
                .filter_map(|(i, j)| {
                    if let Some(mapping) = &self.mapping {
                        if let Some(land) = mapping.get(&("l", i, j)) {
                            let map = mapping.get(&("m", i, j)).unwrap();
                            let env = mapping.get(&("e", i, j)).copied();
                            let xtea = self.inner.xteas().as_ref().unwrap().get(&(((i as u32) << 8) | j as u32));
                            MapSquare::new_named(&self.inner, xtea.copied(), *land, *map, env, i, j).ok()
                        } else {
                            None
                        }
                    } else {
                        MapSquare::new(&self.inner, i, j).ok()
                    }
                })
                .map(|sq| ((sq.i, sq.j), sq))
                .collect();
            GroupMapSquare { core_i, core_j, mapsquares }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.state.size_hint()
    }
}

impl ExactSizeIterator for GroupMapSquareIterator {}
