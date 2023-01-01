//! Describes the properties of <Name>s.

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};
use rs3cache_backend::error::CacheError;
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*};
use serde::Serialize;

use bytes::{Buf, Bytes};
use crate::{
    cache::{buf::{BufExtra,Buffer}, error::CacheResult, index::CacheIndex, indextype::IndexType},
    structures::paramtable::ParamTable,
};

/// Describes the properties of a given <Name>.
#[cfg_eval]
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
 #[cfg_attr(feature = "pyo3", pyclass(frozen))]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct <Name> {
    /// Its id.
    pub id: u32,
   
    #[serde(flatten)]
    pub params: Option<ParamTable>,
}

impl <Name> {
    /// Returns a mapping of all [`<Name>`]s.
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::OBJ_CONFIG, config.input.clone())?.into_iter();

        let <Name>s = archives
            .flat_map(|archive| {
                let archive_id = archive.archive_id();
                archive
                    .take_files()
                    .into_iter()
                    .map(move |(file_id, file)| (archive_id << 8 | file_id, file))
            })
            .map(|(id, file)| (id, Self::deserialize(id, file)))
            .collect::<BTreeMap<u32, Self>>();
        Ok(<Name>s)
    }

    fn deserialize(id: u32, mut buffer: Bytes) -> Self {
        
        let mut <Name> = Self { id, ..Default::default() };

        loop {
            match buffer.get_u8() {
                0 => {
                    debug_assert!(!buffer.has_remaining());
                    break <Name>;
                }
                249 => <Name>.params = Some(ParamTable::deserialize(&mut buffer)),

                missing => unimplemented!("<Name>::deserialize cannot deserialize opcode {} in id {}", missing, id),
            }
        }
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for <Name> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl <Name> {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<Name>({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("<Name>({})", serde_json::to_string(self).unwrap()))
    }
}

/// Save the <Name> configs as `<Name>>.json`. Exposed as `--dump <Name>`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output).map_err(|e| CacheError::io(e, config.output.to_path_buf()))?;
    let mut <Name>_configs = <Name>::dump_all(config)?.into_values().collect::<Vec<_>>();
    <Name>_configs.sort_unstable_by_key(|loc| loc.id);

    let path = path!(config.output / "<Name>_configs.json");

    let mut file = File::create(&path).map_err(|e| CacheError::io(e, path.clone()))?;
    let data = serde_json::to_string_pretty(&<Name>_configs).unwrap();
    file.write_all(data.as_bytes()).map_err(|e| CacheError::io(e, path))?;

    Ok(())
}
