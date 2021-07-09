#![cfg(feature = "osrs")]

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};
use std::path::Path;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
    cache::{buf::  Buffer, index::CacheIndex, indextype::IndexType},
    structures::paramtable::ParamTable,
    utils::error::CacheResult,
};

/// Describes the properties of a given item.
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[skip_serializing_none]
#[derive(Serialize, Debug, Default)]
pub struct TextureConfig {
    /// Its id.
    pub id: u32,
    pub field1777: u16,
    pub field1778: bool,


}

impl TextureConfig {
    /// Returns a mapping of all [`TextureConfig`]s.
    pub fn dump_all() -> CacheResult<HashMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::TEXTURES)?.into_iter();
        let locations = archives
            .flat_map(|archive| {
                let archive_id = archive.archive_id();
                archive
                    .take_files()
                    .into_iter()
            })
            .map(|(id, file)| (id, Self::deserialize(id, file)))
            .collect::<HashMap<u32, Self>>();
        Ok(locations)
    }

    fn deserialize(id: u32, file: Vec<u8>) -> Self {
        let mut buffer =  Buffer::new(file);
        
        let field1777 = buffer.read_unsigned_short();
        let field1778 = buffer.read_byte() != 0;
        let count = buffer.read_unsigned_byte();
        

        
        
       
        let texture = Self{id, field1777, field1778};
        dbg!(&texture);
       
        texture
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for TextureConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

/// Save the textures as `textures.json`. Exposed as `--dump item_configs`.
pub fn export(path: impl AsRef<Path>) -> CacheResult<()> {
    let path = path.as_ref();

    fs::create_dir_all(path)?;
    let mut loc_configs = TextureConfig::dump_all()?.into_values().collect::<Vec<_>>();
    loc_configs.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(format!("{}.json", path.to_str().unwrap()))?;
    let data = serde_json::to_string_pretty(&loc_configs).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}

