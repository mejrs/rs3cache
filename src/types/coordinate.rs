//! The [`Coordinate`] type.

use std::convert::TryFrom;

use serde::Serialize;

/// A coordinate.
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, Serialize, PartialOrd, Ord, PartialEq, Eq)]
pub struct Coordinate {
    pub plane: u8,
    pub x: u16,
    pub y: u16,
}

impl TryFrom<u32> for Coordinate {
    type Error = &'static str;

    fn try_from(i: u32) -> Result<Self, Self::Error> {
        let plane = (i >> 28) as u8;
        let x = ((i >> 14) & 0x3FFF) as u16;
        let y = (i & 0x3FFF) as u16;

        if (plane > 3) | (x > 6400) | (y > 12800) {
            Err("invalid coordinate")
        } else {
            Ok(Self { plane, x, y })
        }
    }
}
