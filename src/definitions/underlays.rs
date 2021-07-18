use std::collections::HashMap;
use std::path::Path;
use std::fs::{self, File};
use std::io::Write;

use crate::{
    cache::{
        buf::  Buffer,
        index::CacheIndex,
        indextype::{ConfigType, IndexType},
    },
    utils::error::CacheResult,
};
use serde::Serialize;
use serde_with::skip_serializing_none;

/// Describes the general ground colour. This colour is blended with surrounding tiles.
#[cfg_attr(feature = "pyo3", pyclass)]
#[skip_serializing_none]
#[derive(Debug, Default, Copy, Clone, Serialize)]

pub struct Underlay {
    /// Id of the underlay configuration.
    pub id: u32,
    /// Ground colour of this tile type
    pub colour: Option<(u8, u8, u8)>,
    #[cfg(feature = "rs3")]
    op_2: Option<u16>,
    #[cfg(feature = "rs3")]
    op_3: Option<u16>,
    #[cfg(feature = "rs3")]
    op_4: Option<bool>,
    #[cfg(feature = "rs3")]
    op_5: Option<bool>,
}

impl Underlay {
    /// Returns a mapping of all [`Underlay`] configurations.
    pub fn dump_all() -> CacheResult<HashMap<u32, Underlay>> {
        Ok(CacheIndex::new(IndexType::CONFIG)?
            .archive(ConfigType::UNDERLAYS)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, Underlay::deserialize(file_id, file)))
            .collect())
    }

    fn deserialize(id: u32, file: Vec<u8>) -> Underlay {
        let mut buffer =  Buffer::new(file);
        let mut underlay = Underlay { id, ..Default::default() };

        loop {
            let opcode = buffer.read_unsigned_byte();
            match opcode {
                0 => {
                    assert_eq!(buffer.remaining(), 0);
                    break underlay;
                }
                1 => underlay.colour = Some(buffer.read_rgb()),
                #[cfg(feature = "rs3")]
                2 => underlay.op_2 = Some(buffer.read_unsigned_short()),
                #[cfg(feature = "rs3")]
                3 => underlay.op_3 = Some(buffer.read_unsigned_short()),
                #[cfg(feature = "rs3")]
                4 => underlay.op_4 = Some(true),
                #[cfg(feature = "rs3")]
                5 => underlay.op_5 = Some(true),

                missing => unimplemented!("Underlay::deserialize cannot deserialize opcode {} in id {}", missing, id),
            }
        }
    }
}

/// Save the location configs as `location_configs.json`. Exposed as `--dump location_configs`.
pub fn export(path: impl AsRef<Path>) -> CacheResult<()> {
    let path = path.as_ref();

    fs::create_dir_all(path)?;
    let mut underlay = Underlay::dump_all()?.into_values().collect::<Vec<_>>();
    underlay.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(format!("{}.json", path.to_str().unwrap()))?;
    let data = serde_json::to_string_pretty(&underlay).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}