use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};

use bytes::{Buf, Bytes};
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::cache::{
    buf::BufExtra,
    error::CacheResult,
    index::CacheIndex,
    indextype::{ConfigType, IndexType},
};

/// Describes the general ground colour. This colour is blended with surrounding tiles.
#[cfg_attr(feature = "pyo3", pyclass)]
#[skip_serializing_none]
#[derive(Debug, Default, Copy, Clone, Serialize)]

pub struct Underlay {
    /// Id of the underlay configuration.
    pub id: u32,
    /// Ground colour of this tile type
    pub colour: Option<[u8; 3]>,
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
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Underlay>> {
        Ok(CacheIndex::new(IndexType::CONFIG, &config.input)?
            .archive(ConfigType::UNDERLAYS)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, Underlay::deserialize(file_id, file)))
            .collect())
    }

    fn deserialize(id: u32, mut buffer: Bytes) -> Underlay {
        let mut underlay = Underlay { id, ..Default::default() };

        loop {
            let opcode = buffer.get_u8();
            match opcode {
                0 => {
                    assert!(!buffer.has_remaining());
                    break underlay;
                }
                1 => underlay.colour = Some(buffer.get_rgb()),
                #[cfg(feature = "rs3")]
                2 => underlay.op_2 = Some(buffer.get_u16()),
                #[cfg(feature = "rs3")]
                3 => underlay.op_3 = Some(buffer.get_u16()),
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
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output)?;
    let mut underlay = Underlay::dump_all(config)?.into_values().collect::<Vec<_>>();
    underlay.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(path!(config.output / "underlays.json"))?;
    let data = serde_json::to_string_pretty(&underlay).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}
