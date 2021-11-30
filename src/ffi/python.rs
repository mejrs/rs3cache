//! Python bindings for `rs3cache`.
//!
//! See the README for help with installing this.
//!
//!  # Usage
//!
//! Usage examples are available in the [/rs3cache](https://github.com/mejrs/rs3cache/tree/master/rs3cache "Examples") folder.
//!
//! All the following can be imported as:
//! ```python
//! from rs3cache import *
//! ```
//! ## Functions
//!
//! The following functions are available:
//! ```python
//! get_location_configs()
//! get_npc_configs()
//! get_item_configs()
//! get_varbit_configs()
//! get_struct_configs()
//! get_enum_configs()
//! ```
//! ## Classes
//!
//! The following classes are available:
//! ```python
//! MapSquares
//! CacheIndex
//! ```

#![cfg(feature = "pyo3")]

mod index;
mod mapsquares;
mod sprites;

use std::{collections::BTreeMap, path::PathBuf};

pub use index::*;
pub use mapsquares::*;
use pyo3::{prelude::*, wrap_pyfunction};
pub use sprites::*;

#[cfg(feature = "rs3")]
use crate::definitions::achievements::Achievement;
use crate::{
    cli::Config,
    definitions::{
        enums::Enum, item_configs::ItemConfig, location_configs::LocationConfig, npc_configs::NpcConfig, structs::Struct,
        varbit_configs::VarbitConfig,
    },
};

pub fn initializer(_py: Python, m: &PyModule) -> PyResult<()> {
    #[cfg(feature = "rs3")]
    m.add_function(wrap_pyfunction!(get_achievement_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_location_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_npc_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_item_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_varbit_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_struct_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_enum_configs, m)?)?;

    m.add_class::<PyMapSquares>()?;
    m.add_class::<PyCacheIndex>()?;
    m.add_class::<PySprites>()?;
    Ok(())
}

/// Wrapper for [`Achievement::dump_all`]

#[pyfunction]
#[cfg(feature = "rs3")]
pub fn get_achievement_configs(path: Option<PathBuf>) -> PyResult<BTreeMap<u32, Achievement>> {
    let mut config = Config::env();
    if let Some(path) = path {
        config.input = path
    }
    Ok(Achievement::dump_all(&config)?)
}

/// Wrapper for [`LocationConfig::dump_all`]
#[pyfunction]
pub fn get_location_configs(path: Option<PathBuf>) -> PyResult<BTreeMap<u32, LocationConfig>> {
    let mut config = Config::env();
    if let Some(path) = path {
        config.input = path
    }
    Ok(LocationConfig::dump_all(&config)?)
}

/// Wrapper for [`NpcConfig::dump_all`]
#[pyfunction]
pub fn get_npc_configs() -> PyResult<BTreeMap<u32, NpcConfig>> {
    Ok(NpcConfig::dump_all(&Config::env())?)
}

/// Wrapper for [`NpcConfig::dump_all`]
#[pyfunction]
pub fn get_item_configs() -> PyResult<BTreeMap<u32, ItemConfig>> {
    Ok(ItemConfig::dump_all(&Config::env())?)
}

/// Wrapper for [`Struct::dump_all`]
#[pyfunction]
pub fn get_struct_configs() -> PyResult<BTreeMap<u32, Struct>> {
    Ok(Struct::dump_all(&Config::env())?)
}

/// Wrapper for [`Struct::dump_all`]
#[pyfunction]
pub fn get_enum_configs() -> PyResult<BTreeMap<u32, Enum>> {
    Ok(Enum::dump_all(&Config::env())?)
}

/// Wrapper for [`VarbitConfig::dump_all`]
#[pyfunction]
pub fn get_varbit_configs() -> PyResult<BTreeMap<u32, VarbitConfig>> {
    Ok(VarbitConfig::dump_all(&Config::env())?)
}
