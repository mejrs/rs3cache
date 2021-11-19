use core::ops::{Range, RangeInclusive};
use std::{
    collections::{hash_map, BTreeMap, HashMap},
    iter::Zip,
};

use bytes::{Buf, Bytes};
use itertools::{iproduct, Product};
use ndarray::{iter::LanesIter, s, Axis, Dim};

use crate::{
    cache::{
        arc::Archive,
        index::{self, CacheIndex},
        indextype::IndexType,
    },
    definitions::{
        locations::Location,
        mapsquares::{GroupMapSquare, MapSquare},
        tiles::{Tile, TileArray},
    },
    utils::{
        error::{CacheError, CacheResult},
        rangeclamp::RangeClamp,
    },
};

/// Iterates over all [`MapSquare`]s in arbitrary order.

pub struct MapSquareIterator {
    inner: CacheIndex<index::Initial>,
    mapping: BTreeMap<(&'static str, u8, u8), u32>,
    state: std::vec::IntoIter<(u8, u8)>,
}

impl MapSquareIterator {
    /// Constructor for MapSquareIterator.
    pub fn new() -> CacheResult<Self> {
        todo!()
    }
}

impl Iterator for MapSquareIterator {
    type Item = MapSquare;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
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
    pub fn new(range_i: RangeInclusive<i32>, range_j: RangeInclusive<i32>) -> CacheResult<GroupMapSquareIterator> {
        todo!()
    }
}

impl Iterator for GroupMapSquareIterator {
    type Item = GroupMapSquare;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.state.size_hint()
    }
}

impl ExactSizeIterator for GroupMapSquareIterator {}
