//! Bitpacked player variables.
//!
//!
//! See also [`Varp`](crate::types::variables::Varp) and [`Varbit`](crate::types::variables::Varbit).

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, PyObjectProtocol};
use serde::Serialize;

use crate::{
    cache::{
        buf::  Buffer,
        index::CacheIndex,
        indextype::{ConfigType, IndexType},
    },
    utils::error::CacheResult,
};
/// A varbit configuration.
///
/// The varbit is the bits of Varp `index` from `least_significant_bit` to `most_significant_bit` inclusive.
#[cfg_attr(feature = "pyo3", pyclass)]
#[allow(missing_docs)]
#[derive(Debug, Serialize)]
pub struct VarbitConfig {
    /// Id of the [`Varbit`](crate::types::variables::Varbit).
    pub id: u32,
    pub unknown_1: u8,
    /// The Varp that this varbit maps to.
    pub index: u16,
    pub least_significant_bit: u8,
    pub most_significant_bit: u8,
}

impl VarbitConfig {
    /// Returns a mapping of all [`VarbitConfig`]s.
    pub fn dump_all() -> CacheResult<HashMap<u32, Self>> {
        Ok(CacheIndex::new(IndexType::CONFIG)?
            .archive(ConfigType::VARBITS)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, VarbitConfig::deserialize(file_id, file)))
            .collect())
    }

    fn deserialize(id: u32, file: Vec<u8>) -> Self {
        let mut buffer =  Buffer::new(file);

        let mut unknown_1 = None;
        let mut index = None;
        let mut least_significant_bit = None;
        let mut most_significant_bit = None;

        loop {
            match buffer.read_unsigned_byte() {
                0 => {
                    assert_eq!(buffer.remaining(), 0);
                    break Self {
                        id,
                        unknown_1: unknown_1.expect("opcode 1 was not read"),
                        index: index.expect("opcode 1 was not read"),
                        least_significant_bit: least_significant_bit.expect("opcode 2 was not read"),
                        most_significant_bit: most_significant_bit.expect("opcode 2 was not read"),
                    };
                }
                1 => {
                    unknown_1 = Some(buffer.read_unsigned_byte());
                    index = Some(buffer.read_unsigned_short());
                }
                2 => {
                    least_significant_bit = Some(buffer.read_unsigned_byte());
                    most_significant_bit = Some(buffer.read_unsigned_byte());
                }
                opcode => unimplemented!("unknown varbit_config opcode {}", opcode),
            }
        }
    }
}

/// Save the varbit configs as `varbit_configs.json`. Exposed as `--dump varbit_configs`.
pub fn export() -> CacheResult<()> {
    fs::create_dir_all("out")?;
    let mut vb_configs = VarbitConfig::dump_all()?.into_values().collect::<Vec<_>>();
    vb_configs.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create("out/varbit_configs.json")?;
    let data = serde_json::to_string_pretty(&vb_configs).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl VarbitConfig {
    #[getter]
    fn id(&self) -> PyResult<u32> {
        Ok(self.id)
    }
    #[getter]
    fn unknown_1(&self) -> PyResult<u8> {
        Ok(self.unknown_1)
    }
    #[getter]
    fn index(&self) -> PyResult<u16> {
        Ok(self.index)
    }
    #[getter]
    fn least_significant_bit(&self) -> PyResult<u8> {
        Ok(self.most_significant_bit)
    }
    #[getter]
    fn most_significant_bit(&self) -> PyResult<u8> {
        Ok(self.most_significant_bit)
    }
}

#[cfg(feature = "pyo3")]
#[pyproto]
impl PyObjectProtocol for VarbitConfig {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("VarbitConfig({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("VarbitConfig({})", serde_json::to_string(self).unwrap()))
    }
}
