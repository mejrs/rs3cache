use std::{ffi::OsStr, fmt, path::PathBuf, sync::Arc};

use clap::{ArgEnum, Parser};
use rs3cache_backend::{error::CacheResult, index::CachePath};

use crate::definitions;
#[cfg(not(target_arch = "wasm32"))]
use crate::renderers::map;

#[cfg(not(target_arch = "wasm32"))]
#[derive(ArgEnum, Clone, Debug)]
#[clap(rename_all = "snake_case")]
pub enum Render {
    All,
    Map,
}

#[cfg(not(target_arch = "wasm32"))]
impl Render {
    pub fn call(&self, config: &Config) -> CacheResult<()> {
        match self {
            Render::All => map::render(config)?,
            Render::Map => map::render(config)?,
        };

        Ok(())
    }
}

#[derive(ArgEnum, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[clap(rename_all = "snake_case")]
pub enum Dump {
    All,
    Configs,
    #[cfg(feature = "rs3")]
    Music,
    #[cfg(feature = "rs3")]
    Achievements,
    Sprites,
    Locations,
    LocationsEach,
    TilesEach,
    LocationConfigs,
    LocationConfigsEach,
    NpcConfig,
    ItemConfigs,
    Maplabels,
    #[cfg(feature = "rs3")]
    Worldmaps,
    VarbitConfigs,
    Structs,
    Enums,
    #[cfg(any(feature = "rs3", feature = "osrs"))]
    Underlays,
    #[cfg(any(feature = "rs3", feature = "osrs"))]
    Overlays,
    #[cfg(feature = "osrs")]
    Textures,
}

impl Dump {
    pub fn call(&self) -> fn(&Config) -> CacheResult<()> {
        match self {
            #[cfg(feature = "rs3")]
            Dump::Music => definitions::music::export_each,
            #[cfg(feature = "rs3")]
            Dump::Achievements => definitions::achievements::export,
            Dump::Sprites => definitions::sprites::save_all,
            Dump::TilesEach => definitions::mapsquares::export_tiles_by_square,
            Dump::Locations => definitions::mapsquares::export_locations_by_id,
            Dump::LocationsEach => definitions::mapsquares::export_locations_by_square,
            Dump::LocationConfigs => definitions::location_configs::export,
            Dump::LocationConfigsEach => definitions::location_configs::export_each,
            Dump::NpcConfig => definitions::npc_configs::export,
            Dump::ItemConfigs => definitions::item_configs::export,
            Dump::Maplabels => definitions::maplabel_configs::export,
            #[cfg(feature = "rs3")]
            Dump::Worldmaps => |config| try {
                definitions::worldmaps::dump_big(config)?;
                definitions::worldmaps::dump_small(config)?;
                definitions::worldmaps::export_pastes(config)?;
                definitions::worldmaps::export_zones(config)?;
            },
            Dump::VarbitConfigs => definitions::varbit_configs::export,
            Dump::Structs => definitions::structs::export,
            Dump::Enums => definitions::enums::export,
            #[cfg(any(feature = "rs3", feature = "osrs"))]
            Dump::Underlays => definitions::underlays::export,
            #[cfg(any(feature = "rs3", feature = "osrs"))]
            Dump::Overlays => definitions::overlays::export,
            #[cfg(feature = "osrs")]
            Dump::Textures => definitions::textures::export,
            Dump::All | Dump::Configs => |_| Ok(()),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            #[cfg(feature = "rs3")]
            Dump::Music => "music",
            #[cfg(feature = "rs3")]
            Dump::Achievements => "achievements",
            Dump::Sprites => "sprites",
            Dump::TilesEach => "tiles_by_square",
            Dump::Locations => "locations_by_id",
            Dump::LocationsEach => "locations_by_square",
            Dump::LocationConfigs => "location_configs",
            Dump::LocationConfigsEach => "location_configs_each",
            Dump::NpcConfig => "npc_configs",
            Dump::ItemConfigs => "item_configs",
            Dump::Maplabels => "maplabel_configs",
            #[cfg(feature = "rs3")]
            Dump::Worldmaps => "world_maps",
            Dump::VarbitConfigs => "varbit_configs",
            Dump::Structs => "structs",
            Dump::Enums => "enums",
            #[cfg(any(feature = "rs3", feature = "osrs"))]
            Dump::Underlays => "underlays",
            #[cfg(any(feature = "rs3", feature = "osrs"))]
            Dump::Overlays => "overlays",
            #[cfg(feature = "osrs")]
            Dump::Textures => "textures",
            Dump::All => "all",
            Dump::Configs => "configs",
        }
    }

    pub fn configs() -> &'static [Self] {
        &[
            #[cfg(feature = "rs3")]
            Dump::Achievements,
            Dump::LocationConfigs,
            Dump::NpcConfig,
            Dump::ItemConfigs,
            Dump::Maplabels,
            Dump::VarbitConfigs,
            Dump::Structs,
            Dump::Enums,
            #[cfg(any(feature = "rs3", feature = "osrs"))]
            Dump::Underlays,
            #[cfg(any(feature = "rs3", feature = "osrs"))]
            Dump::Overlays,
            #[cfg(feature = "osrs")]
            Dump::Textures,
        ]
    }
}

impl fmt::Display for Dump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

fn path_helper(input: &OsStr) -> Arc<CachePath> {
    Arc::new(CachePath::Given(input.into()))
}

const INPUT: &str = if cfg!(feature = "rs3") {
    "RS3_CACHE_INPUT_FOLDER"
} else if cfg!(feature = "osrs") {
    "OSRS_CACHE_INPUT_FOLDER"
} else if cfg!(feature = "legacy") {
    "LEGACY_CACHE_INPUT_FOLDER"
} else {
    unimplemented!()
};

const OUTPUT: &str = if cfg!(feature = "rs3") {
    "RS3_CACHE_OUTPUT_FOLDER"
} else if cfg!(feature = "osrs") {
    "OSRS_CACHE_OUTPUT_FOLDER"
} else if cfg!(feature = "legacy") {
    "LEGACY_CACHE_OUTPUT_FOLDER"
} else {
    unimplemented!()
};

#[derive(Debug, Default, Parser)]
#[clap(author, about = "Tools and api for reading and interpreting the RuneScape game cache")]
pub struct Config {
    /// The path where to look for the current cache.
    #[clap(parse(from_os_str = path_helper), long, env = INPUT, default_value = "")]
    pub input: Arc<CachePath>,

    /// The path where to place output.
    #[clap(long, env = OUTPUT, default_value = "")]
    pub output: PathBuf,

    /// This exports them as small tiles, formatted as `<layer>/<mapid>/<zoom>/<plane>_<x>_<y>.png`,
    /// suitable for use with interactive map libraries such as <https://leafletjs.com/>,
    /// as seen on <https://mejrs.github.io/>
    #[cfg(not(target_arch = "wasm32"))]
    #[clap(arg_enum, long, multiple_values = true)]
    pub render: Vec<Render>,

    /// Dumps the given archives.
    #[clap(arg_enum, long, multiple_values = true)]
    pub dump: Vec<Dump>,

    /// Checks whether the cache is in a consistent state.
    /// Indices 14, 40, 54, 55 are not necessarily complete.
    #[clap(long)]
    pub assert_coherence: bool,
}

impl Config {
    #[cfg(not(feature = "mockdata"))]
    pub fn env() -> Self {
        Self {
            input: Arc::new(CachePath::Env(std::env::var_os(INPUT).unwrap_or_default().into())),
            output: std::env::var_os(OUTPUT).unwrap_or_default().into(),
            ..Default::default()
        }
    }

    #[cfg(all(feature = "osrs", feature = "mockdata"))]
    pub fn env() -> Self {
        Self {
            input: Arc::new(CachePath::Given(PathBuf::from("test_data/osrs_cache"))),
            ..Default::default()
        }
    }

    #[cfg(all(feature = "rs3", feature = "mockdata"))]
    pub fn env() -> Self {
        Self {
            input: Arc::new(CachePath::Given(PathBuf::from("test_data/rs3_cache"))),
            ..Default::default()
        }
    }

    #[cfg(all(feature = "legacy", feature = "mockdata"))]
    pub fn env() -> Self {
        Self {
            input: Arc::new(CachePath::Given(PathBuf::from("test_data/2005_cache"))),
            ..Default::default()
        }
    }
}
