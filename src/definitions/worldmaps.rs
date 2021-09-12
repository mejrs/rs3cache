//!

use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::Write,
    iter,
};

use fstrings::{f, format_args_f};
use path_macro::path;
use serde::Serialize;

use crate::{
    cache::{buf::Buffer, error::CacheResult, index::CacheIndex, indextype::IndexType},
    types::coordinate::Coordinate,
};

/// Enumeration of the archives in the [WORLDMAP](IndexType::WORLDMAP) index.
pub struct WorldMapType;

impl WorldMapType {
    #![allow(missing_docs)]
    pub const ZONES: u32 = 0;
    pub const PASTES: u32 = 1;

    /// Used to draw the minimap in the top left of the ingame world map interface.
    pub const SMALL: u32 = 2;

    pub const UNKNOWN_3: u32 = 3;
    pub const BIG: u32 = 4;
}
/// Describes the general properties of a map zone.
#[derive(Debug, Serialize)]
pub struct MapZone {
    id: u32,
    internal_name: String,
    name: String,
    center: Coordinate,
    unknown_1: u32,
    show: bool,
    default_zoom: u8,
    unknown_2: u8,
    bounds: Vec<BoundDef>,
}

impl MapZone {
    /// Returns a mapping of all [`MapZone`] configurations.
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<HashMap<u32, Self>> {
        Ok(CacheIndex::new(IndexType::WORLDMAP, &config.input)?
            .archive(WorldMapType::ZONES)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, Self::deserialize(file_id, file)))
            .collect())
    }

    fn deserialize(id: u32, file: Vec<u8>) -> Self {
        let mut buf = Buffer::new(file);
        let internal_name = buf.read_string();
        let name = buf.read_string();
        let center = buf.read_unsigned_int().try_into().unwrap();
        let unknown_1 = buf.read_unsigned_int();
        let show = match buf.read_unsigned_byte() {
            0 => false,
            1 => true,
            other => unimplemented!("Cannot convert value {} for 'show' to boolean", other),
        };
        let default_zoom = buf.read_unsigned_byte();
        let unknown_2 = buf.read_unsigned_byte();
        let count = buf.read_unsigned_byte() as usize;
        let bounds = iter::repeat_with(|| BoundDef::deserialize(&mut buf)).take(count).collect();

        debug_assert_eq!(buf.remaining(), 0);

        Self {
            id,
            internal_name,
            name,
            center,
            unknown_1,
            show,
            default_zoom,
            unknown_2,
            bounds,
        }
    }

    /// Get a reference to the map zone's internal name.
    pub fn internal_name(&self) -> &str {
        &self.internal_name
    }

    /// Get a reference to the map zone's internal name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the map zone's center coordinate.
    pub const fn center(&self) -> Coordinate {
        self.center
    }

    /// Whether the mapzone is shown.
    pub const fn show(&self) -> bool {
        self.show
    }

    /// Get a reference to the map zone's unknown 1.
    pub const fn unknown_1(&self) -> u32 {
        self.unknown_1
    }

    /// The map zone's default zoom.
    pub const fn default_zoom(&self) -> u8 {
        self.default_zoom
    }

    /// Get a reference to the map zone's unknown 2.
    pub const fn unknown_2(&self) -> u8 {
        self.unknown_2
    }

    /// Get a reference to the map zone's bounds.
    pub fn bounds(&self) -> &[BoundDef] {
        self.bounds.as_slice()
    }
}

mod mapzone_fields_impl {
    #![allow(missing_docs)]
    use serde::Serialize;

    use crate::cache::buf::Buffer;

    #[derive(Debug, Serialize)]
    pub struct BoundDef {
        plane: u8,
        src: Bound,
        dst: Bound,
    }

    impl BoundDef {
        pub fn deserialize(buf: &mut Buffer<Vec<u8>>) -> Self {
            let plane = buf.read_unsigned_byte();
            let src = Bound::deserialize(buf);
            let dst = Bound::deserialize(buf);
            Self { plane, src, dst }
        }
    }

    /// Represents a rectangular area of the game map..
    #[derive(Debug, Serialize)]
    pub struct Bound {
        pub west: u16,
        pub south: u16,
        pub east: u16,
        pub north: u16,
    }

    impl Bound {
        pub fn deserialize(buf: &mut Buffer<Vec<u8>>) -> Self {
            let west = buf.read_unsigned_short();
            let south = buf.read_unsigned_short();
            let east = buf.read_unsigned_short();
            let north = buf.read_unsigned_short();

            Self { west, south, east, north }
        }
    }
}

pub use mapzone_fields_impl::*;

/// Describes how a worldmap is formed from the actual map.
#[derive(Debug, Serialize)]
pub struct MapPastes {
    ///The map id.
    pub id: u32,
    /// The horizontal dimension of the world map.
    pub dim_i: u8,
    /// The vertical dimension of the world map.
    pub dim_j: u8,

    /// The [`Paste`]s making up the world map.
    pub pastes: Vec<Paste>,
}

impl MapPastes {
    /// Returns a mapping of all [`MapPastes`].
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<HashMap<u32, Self>> {
        Ok(CacheIndex::new(IndexType::WORLDMAP, &config.input)?
            .archive(WorldMapType::PASTES)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, Self::deserialize(file_id, file)))
            .collect())
    }

    /// Constructor for [`MapPastes`].
    pub fn deserialize(id: u32, file: Vec<u8>) -> Self {
        let mut buf = Buffer::new(file);
        let mut pastes = Vec::new();

        let square_count = buf.read_unsigned_short() as usize;
        let square_pastes = iter::repeat_with(|| Paste::deserialize_square(&mut buf)).take(square_count);
        pastes.extend(square_pastes);

        let chunk_count = buf.read_unsigned_short() as usize;
        let chunk_pastes = iter::repeat_with(|| Paste::deserialize_chunk(&mut buf)).take(chunk_count);
        pastes.extend(chunk_pastes);
        let dim_i = buf.read_unsigned_byte();
        let dim_j = buf.read_unsigned_byte();
        debug_assert_eq!(buf.remaining(), 0);

        Self { id, dim_i, dim_j, pastes }
    }
}

mod mappaste_fields_impl {
    #![allow(missing_docs)]
    use serde::Serialize;

    use crate::cache::buf::Buffer;

    #[derive(Debug, Serialize)]
    pub struct Paste {
        pub src_plane: u8,
        pub n_planes: u8,
        pub src_i: u16,
        pub src_j: u16,
        pub src_chunk: Option<Chunk>,

        pub dst_plane: u8,
        pub dst_i: u16,
        pub dst_j: u16,

        pub dst_chunk: Option<Chunk>,
    }

    impl Paste {
        pub fn deserialize_square(buf: &mut Buffer<Vec<u8>>) -> Self {
            let src_plane = buf.read_unsigned_byte();
            let n_planes = buf.read_unsigned_byte();
            let src_i = buf.read_unsigned_short();
            let src_j = buf.read_unsigned_short();

            let dst_plane = buf.read_unsigned_byte();
            let dst_i = buf.read_unsigned_short();
            let dst_j = buf.read_unsigned_short();

            Self {
                src_plane,
                n_planes,
                src_i,
                src_j,
                src_chunk: None,

                dst_plane,
                dst_i,
                dst_j,

                dst_chunk: None,
            }
        }

        pub fn deserialize_chunk(buf: &mut Buffer<Vec<u8>>) -> Self {
            let src_plane = buf.read_unsigned_byte();
            let n_planes = buf.read_unsigned_byte();
            let src_i = buf.read_unsigned_short();
            let src_j = buf.read_unsigned_short();
            let src_chunk = Chunk::deserialize(buf);

            let dst_plane = buf.read_unsigned_byte();
            let dst_i = buf.read_unsigned_short();
            let dst_j = buf.read_unsigned_short();
            let dst_chunk = Chunk::deserialize(buf);

            Self {
                src_plane,
                n_planes,
                src_i,
                src_j,
                src_chunk: Some(src_chunk),

                dst_plane,
                dst_i,
                dst_j,

                dst_chunk: Some(dst_chunk),
            }
        }
    }

    #[derive(Debug, Serialize)]
    pub struct Chunk {
        pub x: u8,
        pub y: u8,
    }

    impl Chunk {
        pub fn deserialize(buf: &mut Buffer<Vec<u8>>) -> Self {
            let x = buf.read_unsigned_byte();
            let y = buf.read_unsigned_byte();
            Self { x, y }
        }
    }
}
pub use mappaste_fields_impl::*;

/// Exports all world map pastes to `out/map_pastes.json`.
pub fn export_pastes(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output)?;
    // btreemap has deterministic order
    let map_pastes: BTreeMap<u32, MapPastes> = MapPastes::dump_all(config)?.into_iter().collect();

    let mut file = File::create(path!(config.output / "map_pastes.json"))?;
    let data = serde_json::to_string_pretty(&map_pastes)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

/// Exports all world map zones to `out/map_zones.json`.
pub fn export_zones(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output)?;

    let mut map_zones = MapZone::dump_all(config)?.into_values().collect::<Vec<_>>();
    map_zones.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(path!(config.output / "map_zones.json"))?;
    let data = serde_json::to_string_pretty(&map_zones)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

/// Exports small images of world maps to `out/world_map_small`.
pub fn dump_small(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(path!(config.output / "world_map_small"))?;

    let files = CacheIndex::new(IndexType::WORLDMAP, &config.input)?
        .archive(WorldMapType::SMALL)?
        .take_files();
    for (id, data) in files {
        let filename = path!(config.output / "world_map_small" / f!("{id}.png"));
        let mut file = File::create(filename)?;
        file.write_all(&data)?;
    }

    Ok(())
}

/// Exports big images of world maps to `out/world_map_big`.
pub fn dump_big(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(path!(config.output / "world_map_big"))?;

    let files = CacheIndex::new(IndexType::WORLDMAP, &config.input)?
        .archive(WorldMapType::BIG)?
        .take_files();

    for (id, data) in files {
        let mut buf = Buffer::new(data);
        let size = buf.read_unsigned_int() as usize;
        let img = buf.read_n_bytes(size);

        let filename = path!(config.output / "world_map_big" / f!("{id}.png"));
        let mut file = File::create(filename)?;
        file.write_all(&img)?;
    }

    Ok(())
}
