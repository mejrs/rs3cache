#![doc = include_str!("../README.md")]
#![feature(
    cfg_eval,
    option_result_contains,
    option_get_or_insert_default,
    once_cell,
    bench_black_box,
    doc_cfg,
    result_cloned
)]
#![allow(non_snake_case, unused_imports, unreachable_code, unused_variables, dead_code)]
#![warn(
    unused_qualifications,
    unused_import_braces,
    unused_extern_crates,
    rustdoc::broken_intra_doc_links,
    //missing_docs,
    rustdoc::missing_crate_level_docs,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub
)]
#![deny(keyword_idents, macro_use_extern_crate, non_ascii_idents)]

#[cfg(not(any(feature = "rs3", feature = "osrs", feature = "legacy")))]
compile_error!("You must use one and only one of the rs3 or osrs");

#[cfg(all(feature = "mockdata", feature = "save_mockdata"))]
compile_error!("mockdata and save_mockdata are incompatible");

#[cfg(any(feature = "rs3", feature = "osrs", feature = "legacy"))]
pub mod cli;

/// Tools for decoding the cache itself.
#[cfg(any(feature = "rs3", feature = "osrs", feature = "legacy"))]
pub use rs3cache_core as cache;
use rs3cache_utils as utils;

/// Various data types
#[cfg(any(feature = "rs3", feature = "osrs", feature = "legacy"))]
pub mod types {
    pub mod coordinate;
    /// Player variables
    pub mod variables;
}

/// Entities that can be deserialized from cache data.
#[cfg(any(feature = "rs3", feature = "osrs", feature = "legacy"))]
pub mod definitions {
    /// Configurations of Achievements
    #[cfg(feature = "rs3")]
    pub mod achievements;

    /// Configuration of game locations.
    pub mod location_configs;

    /// Describes the id, position, type and rotation of game objects.
    pub mod locations;

    pub mod item_configs;

    /// Configuration of npcs.
    pub mod npc_configs;

    /// Describes the position and id of npcs.
    pub mod npcs;

    /// Describes text, sprites and polygons drawn on the map.
    pub mod maplabel_configs;

    /// Configuration of images drawn on the world map.
    pub mod mapscenes;

    pub mod mapsquares;

    #[cfg(feature = "rs3")]
    pub mod music;

    /// Describes the colours of tiles.
    pub mod overlays;
    /// Images displayed by the game client.
    pub mod sprites;

    pub mod enums;
    pub mod structs;
    #[cfg(feature = "osrs")]
    pub mod textures;

    /// Describes the properties of game surface tiles.
    pub mod tiles;
    /// Describes the ground colours of tiles.
    pub mod underlays;

    pub mod varbit_configs;

    #[cfg(feature = "rs3")]
    pub mod worldmaps;
}

/// Functions for rendering the map.
#[cfg(all(not(target_arch = "wasm32"), any(feature = "rs3", feature = "osrs", feature = "legacy")))]
pub mod renderers {
    /// Exports map tiles.
    pub mod map;

    pub mod scale;

    /// Creates successive tiles for different zoom levels,
    /// for use with a [leaflet.js](https://leafletjs.com/) based map.
    pub mod zoom;
}

/// Contains structures that are used in multiple different configs.
#[cfg(any(feature = "rs3", feature = "osrs", feature = "legacy"))]
pub mod structures {
    /// A mapping of keys to properties.
    pub mod paramtable;
}

/// Foreign function interfaces to `rs3cache`.
#[cfg(any(feature = "rs3", feature = "osrs", feature = "legacy"))]
pub mod ffi {
    pub mod python;
}
