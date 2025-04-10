use core::ops::RangeInclusive;
use std::collections::HashMap;

use itertools::iproduct;
use rs3cache_backend::{
    error::CacheResult,
    index::{self, CacheIndex},
};

use crate::definitions::{
    indextype::IndexType,
    mapsquares::{GroupMapSquare, MapSquare, MapSquares},
};
impl MapSquares {
    pub fn new(config: &crate::cli::Config) -> CacheResult<MapSquares> {
        let index = CacheIndex::new(IndexType::MAPSV2, config.input.clone())?;

        Ok(MapSquares { index })
    }

    pub fn get(&self, i: u8, j: u8) -> CacheResult<MapSquare> {
        let archive_id = (i as u32) | (j as u32) << 7;
        let archive = self.index.archive(archive_id)?;

        Ok(MapSquare::from_archive(archive))
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
    index: CacheIndex<index::Initial>,
    range_i: RangeInclusive<i32>,
    range_j: RangeInclusive<i32>,
    state: std::vec::IntoIter<(u8, u8)>,
}

impl GroupMapSquareIterator {
    pub fn new(range_i: RangeInclusive<i32>, range_j: RangeInclusive<i32>, config: &crate::cli::Config) -> CacheResult<GroupMapSquareIterator> {
        let index = CacheIndex::new(IndexType::MAPSV2, config.input.clone())?;

        let state = index
            .metadatas()
            .keys()
            .map(|id| ((id & 0x7F) as u8, (id >> 7) as u8))
            .collect::<Vec<_>>()
            .into_iter();

        Ok(GroupMapSquareIterator {
            index,
            range_i,
            range_j,
            state,
        })
    }

    #[doc(hidden)]
    pub fn new_only(
        range_i: RangeInclusive<i32>,
        range_j: RangeInclusive<i32>,
        coordinates: Vec<(u8, u8)>,
        config: &crate::cli::Config,
    ) -> CacheResult<GroupMapSquareIterator> {
        let index = CacheIndex::new(IndexType::MAPSV2, config.input.clone())?;

        Ok(GroupMapSquareIterator {
            index,
            range_i,
            range_j,
            state: coordinates.into_iter(),
        })
    }
}

impl Iterator for GroupMapSquareIterator {
    type Item = GroupMapSquare;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.next().map(|(core_i, core_j)| {
            let i = core_i as i32;
            let j = core_j as i32;
            let group_ids = iproduct!(self.range_i.clone(), self.range_j.clone())
                .map(|(di, dj)| (i + di, j + dj))
                .filter(|(i, j)| *i >= 0 && *j >= 0)
                .map(|(i, j)| (i + (j << 7)) as u32);

            let archives = group_ids.filter_map(|archive_id| self.index.archive(archive_id).ok());

            let mapsquares = archives
                .map(MapSquare::from_archive)
                .map(|sq| ((sq.i, sq.j), sq))
                .collect::<HashMap<_, _>>();

            GroupMapSquare { core_i, core_j, mapsquares }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.state.size_hint()
    }
}

impl ExactSizeIterator for GroupMapSquareIterator {}
