use std::{path::PathBuf, str::FromStr};

use clap::{ArgEnum, Args, Parser, Subcommand};
use rs3cache_backend::error::CacheResult;

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

#[derive(ArgEnum, Clone, Debug)]
#[clap(rename_all = "snake_case")]
pub enum Dump {
    All,
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
    pub fn call(&self, config: &Config) -> CacheResult<()> {
        match self {
            Dump::All => {
                #[cfg(feature = "rs3")]
                definitions::achievements::export(config)?;
                #[cfg(feature = "rs3")]
                definitions::npc_configs::export(config)?;
                #[cfg(feature = "rs3")]
                definitions::item_configs::export(config)?;
                definitions::maplabel_configs::export(config)?;
                #[cfg(any(feature = "rs3", feature = "osrs"))]
                definitions::overlays::export(config)?;
                #[cfg(any(feature = "rs3", feature = "osrs"))]
                definitions::underlays::export(config)?;

                #[cfg(feature = "rs3")]
                definitions::worldmaps::dump_big(config)?;
                #[cfg(feature = "rs3")]
                definitions::worldmaps::dump_small(config)?;
                #[cfg(feature = "rs3")]
                definitions::worldmaps::export_pastes(config)?;
                #[cfg(feature = "rs3")]
                definitions::worldmaps::export_zones(config)?;
                #[cfg(feature = "rs3")]
                definitions::varbit_configs::export(config)?;
                #[cfg(feature = "rs3")]
                definitions::structs::export(config)?;
                #[cfg(feature = "rs3")]
                definitions::enums::export(config)?;
                #[cfg(feature = "osrs")]
                definitions::textures::export(config)?;
                definitions::location_configs::export(config)?;

                definitions::sprites::save_all(config)?;
            }
            #[cfg(feature = "rs3")]
            Dump::Music => definitions::music::export_each(config)?,
            #[cfg(feature = "rs3")]
            Dump::Achievements => definitions::achievements::export(config)?,
            Dump::Sprites => definitions::sprites::save_all(config)?,
            Dump::TilesEach => definitions::mapsquares::export_tiles_by_square(config)?,
            Dump::Locations => definitions::mapsquares::export_locations_by_id(config)?,
            Dump::LocationsEach => definitions::mapsquares::export_locations_by_square(config)?,
            Dump::LocationConfigs => definitions::location_configs::export(config)?,
            Dump::LocationConfigsEach => definitions::location_configs::export_each(config)?,
            Dump::NpcConfig => definitions::npc_configs::export(config)?,
            Dump::ItemConfigs => definitions::item_configs::export(config)?,
            Dump::Maplabels => definitions::maplabel_configs::export(config)?,
            #[cfg(feature = "rs3")]
            Dump::Worldmaps => {
                definitions::worldmaps::dump_big(config)?;
                definitions::worldmaps::dump_small(config)?;
                definitions::worldmaps::export_pastes(config)?;
                definitions::worldmaps::export_zones(config)?;
            }
            Dump::VarbitConfigs => definitions::varbit_configs::export(config)?,
            Dump::Structs => definitions::structs::export(config)?,
            Dump::Enums => definitions::enums::export(config)?,
            #[cfg(any(feature = "rs3", feature = "osrs"))]
            Dump::Underlays => definitions::underlays::export(config)?,
            #[cfg(any(feature = "rs3", feature = "osrs"))]
            Dump::Overlays => definitions::overlays::export(config)?,
            #[cfg(feature = "osrs")]
            Dump::Textures => definitions::textures::export(config)?,
        };

        Ok(())
    }
}

#[derive(Debug, Default, Parser)]
#[clap(author, about = "Tools and api for reading and interpreting the RuneScape 3 game cache")]
pub struct Config {
    /// The path where to look for the current cache.
    #[cfg_attr(feature = "rs3", clap(long, env = "RS3_CACHE_INPUT_FOLDER", default_value = ""))]
    #[cfg_attr(feature = "osrs", clap(long, env = "OSRS_CACHE_INPUT_FOLDER", default_value = ""))]
    #[cfg_attr(feature = "legacy", clap(long))]
    pub input: PathBuf,

    /// The path where to place output.
    #[cfg_attr(feature = "rs3", clap(long, env = "RS3_CACHE_OUTPUT_FOLDER", default_value = ""))]
    #[cfg_attr(feature = "osrs", clap(long, env = "OSRS_CACHE_OUTPUT_FOLDER", default_value = ""))]
    #[cfg_attr(feature = "legacy", clap(long))]
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
            #[cfg(feature = "rs3")]
            input: std::env::var_os("RS3_CACHE_INPUT_FOLDER").unwrap_or_default().into(),
            #[cfg(feature = "rs3")]
            output: std::env::var_os("RS3_CACHE_INPUT_FOLDER").unwrap_or_default().into(),

            #[cfg(feature = "osrs")]
            input: std::env::var_os("OSRS_CACHE_INPUT_FOLDER").unwrap_or_default().into(),
            #[cfg(feature = "osrs")]
            output: std::env::var_os("OSRS_CACHE_INPUT_FOLDER").unwrap_or_default().into(),
            ..Default::default()
        }
    }

    #[cfg(all(feature = "osrs", feature = "mockdata"))]
    pub fn env() -> Self {
        Self {
            input: PathBuf::from("test_data/osrs_cache"),
            ..Default::default()
        }
    }

    #[cfg(all(feature = "rs3", feature = "mockdata"))]
    pub fn env() -> Self {
        Self {
            input: PathBuf::from("test_data/rs3_cache"),
            ..Default::default()
        }
    }

    #[cfg(all(feature = "legacy", feature = "mockdata"))]
    pub fn env() -> Self {
        Self {
            input: PathBuf::from("test_data/2005_cache"),
            ..Default::default()
        }
    }
}
