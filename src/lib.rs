//! # rs3cache
//!
//! Tools and api for reading and interpreting the [Runescape 3](https://www.runescape.com/community) game cache.
//!
//!
//! # Setup
//!
//! - `git clone https://github.com/mejrs/rs3-cache`.
//!
//! - Install the [Rust compiler](https://rustup.rs/).
//!
//! - Configure rustup to use the nightly version: `rustup default nightly`.
//!
//! - Navigate to this repository
//!
//! - Compile the executable with `cargo build --release`.
//!
//! - Either:
//!     - Create a system variable named `RUNESCAPE_CACHE_FOLDER` and set its value to where your cache is located.
//!       Typically, this is `%ProgramData%\Jagex\RuneScape`.
//!     - Copy the entire cache and place it in the `raw` folder.
//!
//!  # Usage (executable)
//!
//! The following commands are available:
//!
//! - `target/release/rs3cache.exe --dump all`: save various archives as JSON in the `out` folder.
//! - Use `target/release/rs3cache.exe --dump <archive>` to only dump a specific archive.
//!
//! - `target/release/rs3cache.exe --render map`: render images of the game surface.
//! This exports them as small tiles,  <br>
//! formatted as `<layer>/<zoom>/<plane>_<x>_<y>.png`,
//! suitable for use with interactive map libraries such as [Leaflet](https://leafletjs.com/),
//! as seen on [mejrs.github.io](https://mejrs.github.io/).
//!
//! - `target/release/rs3cache.exe --assert coherence`: checks whether the cache is in a coherent state.
//!
//! - `target/release/rs3cache.exe --help` to see a list of commands
//!

#![feature(
    const_fn,
    option_result_contains,
    map_into_keys_values,
    try_trait,
    backtrace,
    thread_spawn_unchecked,
    once_cell
)]
#![warn(
    unused_imports,
    unused_qualifications,
    unused_import_braces,
    unused_extern_crates,
    broken_intra_doc_links,
    missing_docs,
    missing_crate_level_docs,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub
)]
#![deny(keyword_idents, macro_use_extern_crate, non_ascii_idents)]

/// Tools for decoding the cache itself.
pub mod cache {
    pub mod arc;
    pub mod buf;
    pub mod decoder;
    pub mod index;
    pub mod indextype;
    pub mod meta;
}

/// Various data types
pub mod types {
    pub mod coordinate;
    /// Player variables
    pub mod variables;
}

/// Entities that can be deserialized from cache data.
pub mod definitions {

    /// Configuration of game locations.
    pub mod location_configs;

    /// Describes the id, position, type and rotation of game objects.
    pub mod locations;

    /// Configuration of npcs.
    pub mod npc_configs;

    /// Describes the position and id of npcs.
    pub mod npcs;

    /// Describes text, sprites and polygons drawn on the map.
    pub mod maplabel_configs;

    /// Configuration of images drawn on the world map.
    pub mod mapscenes;

    pub mod mapsquares;

    /// Describes the colours of tiles.
    pub mod overlays;
    /// Images displayed by the game client.
    pub mod sprites;
    /// Describes the properties of game surface tiles.
    pub mod tiles;
    /// Describes the ground colours of tiles.
    pub mod underlays;

    pub mod varbit_configs;

    pub mod worldmaps;
}

/// Functions for rendering the map.
pub mod renderers {
    /// Exports map tiles.
    pub mod map;

    /// Creates successive tiles for different zoom levels,
    /// for use with a [leaflet.js](https://leafletjs.com/) based map.
    pub mod zoom;
}

/// Utilities and helpers used throughout this crate.
pub mod utils {
    /// Adapters for iterators.
    pub mod adapters;
    /// Various colour constants.
    pub mod color;
    /// Contains the CacheError type.
    pub mod error;
    /// Threadpool adapter for iterators.
    pub mod par;
    /// Clamps a [`Range`](std::ops::Range) to a certain interval.
    pub mod rangeclamp;
}

/// Contains structures that are used in multiple different configs.
pub mod structures {
    /// A mapping of keys to properties.
    pub mod paramtable;
}

/// Foreign function interfaces to `rs3cache`.
pub mod ffi {
    pub mod python;
}
