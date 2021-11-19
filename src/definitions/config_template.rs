// A template for adding new configs

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};

use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::Serialize;
use serde_with::skip_serializing_none;

use bytes::{Buf, Bytes};
use crate::cache::{
    buf::{BufExtra,Buffer},
    error::CacheResult,
    index::CacheIndex,
    indextype::{ConfigType, IndexType},
};

#[cfg_attr(feature = "pyo3", pyclass)]
#[skip_serializing_none]
#[derive(Debug, Default, Copy, Clone, Serialize)]

pub struct <Name> {
    /// Id of the <Name>> configuration.
    pub id: u32,
    


}

impl <Name> {
    /// Returns a mapping of all [`<Name>`] configurations.
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, <Name>>> {
        Ok(CacheIndex::new(IndexType::CONFIG, &config.input)?
            .archive(ConfigType::<Name>)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, <Name>::deserialize(file_id, file)))
            .collect())
    }

    fn deserialize(id: u32, mut buffer: Bytes) -> <Name> {
        
        let mut <Name> = <Name> { id, ..Default::default() };

        loop {
            let opcode = buffer.get_u8();
            match opcode {
                0 => {
                    assert!(!buffer.has_remaining());
                    break <Name>;
                }
                1 => <Name>.colour = Some(buffer.get_rgb()),
                #[cfg(feature = "rs3")]
                2 => <Name>.op_2 = Some(buffer.get_u16()),
                #[cfg(feature = "rs3")]
                3 => <Name>.op_3 = Some(buffer.get_u16()),
                #[cfg(feature = "rs3")]
                4 => <Name>.op_4 = Some(true),
                #[cfg(feature = "rs3")]
                5 => <Name>.op_5 = Some(true),

                missing => unimplemented!("<Name>::deserialize cannot deserialize opcode {} in id {}", missing, id),
            }
        }
    }
}

#[cfg(feature = "pyo3")]
#[pyproto]
impl PyObjectProtocol for <Name> {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<Name>({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("<Name>({})", serde_json::to_string(self).unwrap()))
    }
}

/// Save the <Name> configs as `location_configs.json`. Exposed as `--dump <Name>`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output)?;
    let mut <Name> = <Name>::dump_all(config)?.into_values().collect::<Vec<_>>();
    <Name>.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(path!(config.output / "<Name>s.json"))?;
    let data = serde_json::to_string_pretty(&<Name>).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}
