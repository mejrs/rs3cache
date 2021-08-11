use std::{path::PathBuf, str::FromStr};

use fstrings::{f, format_args_f};
use structopt::StructOpt;

use crate::{definitions, renderers::map, utils::error::CacheResult};

#[derive(Debug)]
pub enum Render {
    All,
    Map,
}

impl Render {
    pub fn call(&self, config: &Config) -> CacheResult<()> {
        match self {
            Render::All => map::render(config)?,
            Render::Map => map::render(config)?,
        };

        Ok(())
    }
}

impl FromStr for Render {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Self::All),
            "map" => Ok(Self::Map),
            _ => Err("oops"),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum Dump {
    All,
    Sprites,
    Locations,
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
    Underlays,
    Overlays,
    #[cfg(feature = "osrs")]
    Textures,
}

impl Dump {
    pub fn call(&self, config: &Config) -> CacheResult<()> {
        match self {
            Dump::All => {
                #[cfg(feature = "rs3")]
                definitions::npc_configs::export(config)?;
                #[cfg(feature = "rs3")]
                definitions::item_configs::export(config)?;
                definitions::maplabel_configs::export(config)?;
                definitions::overlays::export(config)?;
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

                definitions::locations::export(config)?;
                definitions::location_configs::export(config)?;
                definitions::location_configs::export_each(config)?;

                definitions::sprites::save_all(config)?;
            }
            Dump::Sprites => definitions::sprites::save_all(config)?,
            Dump::Locations => definitions::locations::export(config)?,
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
            Dump::Underlays => definitions::underlays::export(config)?,
            Dump::Overlays => definitions::overlays::export(config)?,
            #[cfg(feature = "osrs")]
            Dump::Textures => definitions::textures::export(config)?,
        };

        Ok(())
    }
}

impl FromStr for Dump {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Self::All),
            "sprites" => Ok(Self::Sprites),
            "locations" => Ok(Self::Locations),
            "location_configs" => Ok(Self::LocationConfigs),
            "location_configs_each" => Ok(Self::LocationConfigsEach),
            "npc_configs" => Ok(Self::NpcConfig),
            "item_configs" => Ok(Self::ItemConfigs),
            "maplabels" => Ok(Self::Maplabels),
            #[cfg(feature = "rs3")]
            "worldmaps" => Ok(Self::Worldmaps),
            "varbit_configs" => Ok(Self::VarbitConfigs),
            "structs" => Ok(Self::Structs),
            "enums" => Ok(Self::Enums),
            "underlays" => Ok(Self::Underlays),
            "overlays" => Ok(Self::Overlays),
            #[cfg(feature = "osrs")]
            "textures" => Ok(Self::Textures),
            other => Err(f!("{other} is not supported.")),
        }
    }
}

#[derive(Debug, Default, StructOpt)]
pub struct Config {
    /// The path where to look for the current cache.
    /// If omitted this falls back to the
    #[cfg_attr(feature = "rs3", doc = "\"RS3_CACHE_INPUT_FOLDER\"")]
    #[cfg_attr(feature = "osrs", doc = "\"OSRS_CACHE_INPUT_FOLDER\"")]
    ///  environment variable and then to the current folder if not set.
    #[cfg_attr(feature = "rs3", structopt(long, env = "RS3_CACHE_INPUT_FOLDER", default_value = ""))]
    #[cfg_attr(feature = "osrs", structopt(long, env = "OSRS_CACHE_INPUT_FOLDER", default_value = ""))]
    pub input: PathBuf,

    /// The path where to place output.
    /// If omitted this falls back to the
    #[cfg_attr(feature = "rs3", doc = "\"RS3_CACHE_OUTPUT_FOLDER\"")]
    #[cfg_attr(feature = "osrs", doc = "\"OSRS_CACHE_OUTPUT_FOLDER\"")]
    ///  environment variable and then to the current folder if not set.
    #[cfg_attr(feature = "rs3", structopt(long, env = "RS3_CACHE_OUTPUT_FOLDER", default_value = ""))]
    #[cfg_attr(feature = "osrs", structopt(long, env = "OSRS_CACHE_OUTPUT_FOLDER", default_value = ""))]
    pub output: PathBuf,

    /// Allowed values: [all, map]
    ///
    /// This exports them as small tiles, formatted as `<layer>/<mapid>/<zoom>/<plane>_<x>_<y>.png`,
    /// suitable for use with interactive map libraries such as https://leafletjs.com/,
    /// as seen on https://mejrs.github.io/
    #[structopt(long)]
    pub render: Vec<Render>,

    /// Allowed values: [all sprites, locations, location_configs, location_configs_each, npc_configs, item_configs, maplabels, worldmaps, varbit_configs, structs, enums, underlays, overlays]
    ///
    /// Dumps the given archives.
    #[structopt(long)]
    pub dump: Vec<Dump>,

    /// Checks whether the cache is in a consistent state.
    /// Indices 14, 40, 54, 55 are not necessarily complete.
    #[structopt(long)]
    pub assert_coherence: bool,
}

impl Config {
    pub fn env() -> Self {
        Self {
            input: std::env::var_os("RS3CACHE_INPUT_FOLDER").unwrap_or_default().into(),
            output: std::env::var_os("RS3CACHE_INPUT_FOLDER").unwrap_or_default().into(),
            ..Default::default()
        }
    }
}
