//! Describes the properties of structs.

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};

use bytes::{Buf, Bytes};
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rs3cache_backend::error::Context;
use serde::Serialize;

use crate::{
    cache::{error::CacheResult, index::CacheIndex},
    definitions::indextype::IndexType,
    structures::paramtable::ParamTable,
};

/// Describes the properties of a given item.

#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", pyclass(frozen, get_all))]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct Struct {
    /// Its id.
    pub id: u32,

    #[serde(flatten)]
    pub params: Option<ParamTable>,
}

impl Struct {
    /// Returns a mapping of all [`Struct`]s.
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::STRUCT_CONFIG, config.input.clone())?.into_iter();

        let locations = archives
            .map(Result::unwrap)
            .flat_map(|archive| {
                let archive_id = archive.archive_id();
                archive
                    .take_files()
                    .into_iter()
                    .map(move |(file_id, file)| (archive_id << 5 | file_id, file))
            })
            .map(|(id, file)| (id, Self::deserialize(id, file)))
            .collect::<BTreeMap<u32, Self>>();
        Ok(locations)
    }

    fn deserialize(id: u32, mut buffer: Bytes) -> Self {
        let mut r#struct = Self { id, ..Default::default() };

        loop {
            match buffer.get_u8() {
                0 => {
                    debug_assert!(!buffer.has_remaining());
                    break r#struct;
                }
                249 => r#struct.params = Some(ParamTable::deserialize(&mut buffer)),
                missing => unimplemented!("Struct::deserialize cannot deserialize opcode {} in id {}", missing, id),
            }
        }
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Struct {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Struct({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Struct({})", serde_json::to_string(self).unwrap()))
    }
}

/// Save the item configs as `structs.json`. Exposed as `--dump structs`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output).context(&config.output)?;
    let mut structs = Struct::dump_all(config)?.into_values().collect::<Vec<_>>();
    structs.sort_unstable_by_key(|loc| loc.id);

    let path = path!(&config.output / "structs.json");

    let mut file = File::create(&path).context(path.clone())?;
    let data = serde_json::to_string_pretty(&structs).unwrap();
    file.write_all(data.as_bytes()).context(path)?;

    Ok(())
}
