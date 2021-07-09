//! Describes the properties of structs.

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, PyObjectProtocol};
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
pub struct Struct {
    /// Its id.
    pub id: u32,

    #[serde(flatten)]
    pub params: Option<ParamTable>,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Struct {
    #[getter]
    fn id(&self) -> PyResult<u32> {
        Ok(self.id)
    }

    #[getter]
    fn params(&self) -> PyResult<Option<ParamTable>> {
        Ok(self.params.clone())
    }
}

impl Struct {
    /// Returns a mapping of all [`Struct`]s.
    pub fn dump_all() -> CacheResult<HashMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::STRUCT_CONFIG)?.into_iter();

        let locations = archives
            .flat_map(|archive| {
                let archive_id = archive.archive_id();
                archive
                    .take_files()
                    .into_iter()
                    .map(move |(file_id, file)| (archive_id << 5 | file_id, file))
            })
            .map(|(id, file)| (id, Self::deserialize(id, file)))
            .collect::<HashMap<u32, Self>>();
        Ok(locations)
    }

    fn deserialize(id: u32, file: Vec<u8>) -> Self {
        let mut buffer =  Buffer::new(file);
        let mut r#struct = Self { id, ..Default::default() };

        loop {
            match buffer.read_unsigned_byte() {
                0 => {
                    debug_assert_eq!(buffer.remaining(), 0);
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
#[pyproto]
impl PyObjectProtocol for Struct {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Struct({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Struct({})", serde_json::to_string(self).unwrap()))
    }
}

/// Save the item configs as `structs.json`. Exposed as `--dump structs`.
pub fn export() -> CacheResult<()> {
    fs::create_dir_all("out")?;
    let mut structs = Struct::dump_all()?.into_values().collect::<Vec<_>>();
    structs.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create("out/structs.json")?;
    let data = serde_json::to_string_pretty(&structs).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}
