use std::{path::PathBuf, str::FromStr};

use fstrings::{f, format_args_f};
use structopt::StructOpt;

use crate::{definitions,  renderers::map, utils::error::CacheResult};

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

#[derive(Debug)]
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
    #[structopt(long, env = "RS3CACHE_INPUT_FOLDER", default_value = "")]
    pub input: PathBuf,
    #[structopt(long, env = "RS3CACHE_OUTPUT_FOLDER", default_value = "")]
    pub output: PathBuf,

    #[structopt(long)]
    pub render: Vec<Render>,
    #[structopt(long)]
    pub dump: Vec<Dump>,
    #[structopt(long)]
    pub assertions: Option<String>,
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
