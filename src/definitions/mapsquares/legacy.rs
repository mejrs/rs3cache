use core::ops::RangeInclusive;
use std::collections::{BTreeMap, HashMap};

use itertools::iproduct;
use rs3cache_backend::index::MapsquareMeta;

use crate::{
    cache::{
        error::CacheResult,
        index::{self, CacheIndex},
    },
    definitions::mapsquares::{GroupMapSquare, MapSquare, MapSquares},
};

impl MapSquares {
    pub fn new(config: &crate::cli::Config) -> CacheResult<MapSquares> {
        let mut index = CacheIndex::new(4, config.input.clone())?;
        let meta = index.get_index();
        Ok(MapSquares { index, meta })
    }

    pub fn get(&self, i: u8, j: u8) -> CacheResult<MapSquare> {
        let loc = self.meta[&(i, j)].locfile as u32;
        let map = self.meta[&(i, j)].mapfile as u32;
        let sq = MapSquare::new(&self.index, loc, map, i, j)?;
        Ok(sq)
    }
}

/// Iterates over all [`MapSquare`]s in arbitrary order.
pub struct MapSquareIterator {
    pub(crate) mapsquares: MapSquares,
    pub(crate) state: std::vec::IntoIter<(u8, u8)>,
}

impl MapSquareIterator {
    /// Constructor for MapSquareIterator.
    pub fn new() -> CacheResult<Self> {
        todo!()
    }
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
    meta: BTreeMap<(u8, u8), MapsquareMeta>,
    state: std::vec::IntoIter<(u8, u8)>,
}

impl GroupMapSquareIterator {
    /// Constructor for [`GroupMapSquareIterator`].
    pub fn new(range_i: RangeInclusive<i32>, range_j: RangeInclusive<i32>, config: &crate::cli::Config) -> CacheResult<GroupMapSquareIterator> {
        let mut inner = CacheIndex::new(4, config.input.clone())?;
        let meta = inner.get_index();
        let state = meta.keys().copied().collect::<Vec<_>>().into_iter();
        Ok(GroupMapSquareIterator {
            inner,
            state,
            meta,
            range_i,
            range_j,
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
                    if let Some(meta) = self.meta.get(&(i, j)) {
                        let loc = meta.locfile as u32;
                        let map = meta.mapfile as u32;
                        MapSquare::new(&self.inner, loc, map, i, j).ok()
                    } else {
                        None
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
