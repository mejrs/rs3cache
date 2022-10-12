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

use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

pub use index::*;
pub use mapsquares::*;
use pyo3::{prelude::*, wrap_pyfunction};
use rs3cache_backend::{error::py_error_impl::*, index::CachePath};
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

pub fn initializer(py: Python, m: &PyModule) -> PyResult<()> {
    #[cfg(feature = "rs3")]
    m.add_function(wrap_pyfunction!(get_achievement_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_location_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_npc_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_item_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_varbit_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_struct_configs, m)?)?;
    m.add_function(wrap_pyfunction!(get_enum_configs, m)?)?;
    m.add_function(wrap_pyfunction!(hash_djb2, m)?)?;

    m.add_class::<PyMapSquares>()?;
    m.add_class::<PyCacheIndex>()?;
    m.add_class::<PySprites>()?;

    m.add("CacheNotFoundError", py.get_type::<CacheNotFoundError>())?;
    m.add("ArchiveNotFoundError", py.get_type::<ArchiveNotFoundError>())?;
    m.add("FileMissingError", py.get_type::<FileMissingError>())?;
    #[cfg(feature = "osrs")]
    m.add("XteaError", py.get_type::<XteaError>())?;

    Ok(())
}

/// Wrapper for [`Achievement::dump_all`]

#[pyfunction]
#[cfg(feature = "rs3")]
pub fn get_achievement_configs(path: Option<PathBuf>) -> PyResult<BTreeMap<u32, Achievement>> {
    let mut config = Config::env();
    if let Some(path) = path {
        config.input = Arc::new(CachePath::Given(path))
    }
    Ok(Achievement::dump_all(&config)?)
}

/// Wrapper for [`LocationConfig::dump_all`]
#[pyfunction]
pub fn get_location_configs(path: Option<PathBuf>) -> PyResult<BTreeMap<u32, LocationConfig>> {
    let mut config = Config::env();
    if let Some(path) = path {
        config.input = Arc::new(CachePath::Given(path))
    }
    Ok(LocationConfig::dump_all(&config)?)
}

/// Wrapper for [`NpcConfig::dump_all`]
#[pyfunction]
pub fn get_npc_configs(path: Option<PathBuf>) -> PyResult<BTreeMap<u32, NpcConfig>> {
    let mut config = Config::env();
    if let Some(path) = path {
        config.input = Arc::new(CachePath::Given(path))
    }
    Ok(NpcConfig::dump_all(&config)?)
}

/// Wrapper for [`NpcConfig::dump_all`]
#[pyfunction]
pub fn get_item_configs(path: Option<PathBuf>) -> PyResult<BTreeMap<u32, ItemConfig>> {
    let mut config = Config::env();
    if let Some(path) = path {
        config.input = Arc::new(CachePath::Given(path))
    }
    Ok(ItemConfig::dump_all(&config)?)
}

/// Wrapper for [`Struct::dump_all`]
#[pyfunction]
pub fn get_struct_configs(path: Option<PathBuf>) -> PyResult<BTreeMap<u32, Struct>> {
    let mut config = Config::env();
    if let Some(path) = path {
        config.input = Arc::new(CachePath::Given(path))
    }
    Ok(Struct::dump_all(&config)?)
}

/// Wrapper for [`Struct::dump_all`]
#[pyfunction]
pub fn get_enum_configs(path: Option<PathBuf>) -> PyResult<BTreeMap<u32, Enum>> {
    let mut config = Config::env();
    if let Some(path) = path {
        config.input = Arc::new(CachePath::Given(path))
    }
    Ok(Enum::dump_all(&config)?)
}

/// Wrapper for [`VarbitConfig::dump_all`]
#[pyfunction]
pub fn get_varbit_configs(path: Option<PathBuf>) -> PyResult<BTreeMap<u32, VarbitConfig>> {
    let mut config = Config::env();
    if let Some(path) = path {
        config.input = Arc::new(CachePath::Given(path))
    }
    Ok(VarbitConfig::dump_all(&config)?)
}

#[pyfunction]
pub fn hash_djb2(s: &str) -> i32 {
    rs3cache_backend::hash::hash_djb2(s)
}
