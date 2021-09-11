//! Describes the properties of structs.

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};

use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, PyObjectProtocol};
use serde::Serialize;

use crate::{
    cache::{buf::Buffer, index::CacheIndex, indextype::IndexType},
    structures::paramtable::ParamTable,
    cache::error::CacheResult,
};

/// Describes the properties of a given item.
#[cfg_eval]
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
#[cfg_attr(feature = "pyo3", pyclass)]
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
        let archives = CacheIndex::new(IndexType::STRUCT_CONFIG, &config.input)?.into_iter();

        let locations = archives
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

    fn deserialize(id: u32, file: Vec<u8>) -> Self {
        let mut buffer = Buffer::new(file);
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
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output)?;
    let mut structs = Struct::dump_all(config)?.into_values().collect::<Vec<_>>();
    structs.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(path!(&config.output / "structs.json"))?;
    let data = serde_json::to_string_pretty(&structs).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}
