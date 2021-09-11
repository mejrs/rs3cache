use core::ops::RangeInclusive;
use std::collections::{BTreeMap, HashMap};

use itertools::iproduct;

use crate::{
    cache::{
        index::{self, CacheIndex},
        indextype::IndexType,
    },
    definitions::mapsquares::{GroupMapSquare, MapSquare},
    cache::error::CacheResult,
};

/// Iterates over all [`MapSquare`]s in arbitrary order.

pub struct MapSquareIterator {
    pub(crate) mapsquares: crate::definitions::mapsquares::MapSquares,
    pub(crate) state: std::vec::IntoIter<(u8, u8)>,
}

impl Iterator for MapSquareIterator {
    type Item = MapSquare;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.next().map(|(i, j)| self.mapsquares.get(i, j).unwrap())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.state.size_hint()
    }
}

/// Iterates over [`GroupMapSquare`] in arbitrary order.

pub struct GroupMapSquareIterator {
    inner: CacheIndex<index::Initial>,
    range_i: RangeInclusive<i32>,
    range_j: RangeInclusive<i32>,
    mapping: BTreeMap<(&'static str, u8, u8), u32>,
    state: std::vec::IntoIter<(u8, u8)>,
}

impl GroupMapSquareIterator {
    /// Constructor for [`GroupMapSquareIterator`].
    pub fn new(range_i: RangeInclusive<i32>, range_j: RangeInclusive<i32>, config: &crate::cli::Config) -> CacheResult<GroupMapSquareIterator> {
        let inner = CacheIndex::new(IndexType::MAPSV2, &config.input)?;

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
                    if let Some(land) = self.mapping.get(&("l", i, j)) {
                        let map = self.mapping.get(&("m", i, j)).unwrap();
                        let env = self.mapping.get(&("e", i, j)).copied();
                        let xtea = self.inner.xteas().as_ref().unwrap().get(&(((i as u32) << 8) | j as u32));
                        MapSquare::new(&self.inner, xtea.copied(), *land, *map, env, i, j).ok()
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
