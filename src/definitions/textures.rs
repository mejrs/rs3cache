#![cfg(feature = "osrs")]

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

use ::error::Context;
use bytes::{Buf, Bytes};
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::Serialize;

use crate::{
    cache::{
        error::{self, CacheResult},
        index::CacheIndex,
    },
    definitions::indextype::IndexType,
};

/// Describes the properties of a given item.

#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", pyclass(frozen, get_all))]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct TextureConfig {
    /// Its id.
    pub id: u32,
    pub field1777: u16,
    pub field1778: bool,
}

impl TextureConfig {
    /// Returns a mapping of all [`TextureConfig`]s.
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<HashMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::TEXTURES, config.input.clone())?.into_iter();
        let locations = archives
            .flat_map(|archive| archive.unwrap().take_files().into_iter())
            .map(|(id, file)| (id, Self::deserialize(id, file)))
            .collect::<HashMap<u32, Self>>();
        Ok(locations)
    }

    fn deserialize(id: u32, mut buffer: Bytes) -> Self {
        let field1777 = buffer.get_u16();
        let field1778 = buffer.get_i8() != 0;
        let _count = buffer.get_u8();

        // todo: a lot of fields I dont care about

        Self { id, field1777, field1778 }
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for TextureConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

/// Save the textures as `textures.json`. Exposed as `--dump item_configs`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output).with_context(|| error::Io { path: config.output.clone() })?;

    let mut loc_configs = TextureConfig::dump_all(config)?.into_values().collect::<Vec<_>>();
    loc_configs.sort_unstable_by_key(|loc| loc.id);
    let path = path!(config.output / "textures.json");
    let mut file = File::create(&path).with_context(|| error::Io { path: path.clone() })?;
    let data = serde_json::to_string_pretty(&loc_configs).unwrap();
    file.write_all(data.as_bytes()).context(error::Io { path })?;

    Ok(())
}
