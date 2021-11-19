//! Bitpacked player variables.
//!
//!
//! See also [`Varp`](crate::types::variables::Varp) and [`Varbit`](crate::types::variables::Varbit).

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};

use bytes::{Buf, Bytes};
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, PyObjectProtocol};
use serde::Serialize;

use crate::cache::{
    buf::BufExtra,
    error::CacheResult,
    index::CacheIndex,
    indextype::{ConfigType, IndexType},
};
/// A varbit configuration.
///
/// The varbit is the bits of Varp `index` from `least_significant_bit` to `most_significant_bit` inclusive.
#[cfg_eval]
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
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
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        Ok(CacheIndex::new(IndexType::CONFIG, &config.input)?
            .archive(ConfigType::VARBITS)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, VarbitConfig::deserialize(file_id, file)))
            .collect())
    }

    fn deserialize(id: u32, mut buffer: Bytes) -> Self {
        let mut unknown_1 = None;
        let mut index = None;
        let mut least_significant_bit = None;
        let mut most_significant_bit = None;

        loop {
            match buffer.get_u8() {
                0 => {
                    assert!(!buffer.has_remaining());
                    break Self {
                        id,
                        unknown_1: unknown_1.expect("opcode 1 was not read"),
                        index: index.expect("opcode 1 was not read"),
                        least_significant_bit: least_significant_bit.expect("opcode 2 was not read"),
                        most_significant_bit: most_significant_bit.expect("opcode 2 was not read"),
                    };
                }
                1 => {
                    unknown_1 = Some(buffer.get_u8());
                    index = Some(buffer.get_u16());
                }
                2 => {
                    least_significant_bit = Some(buffer.get_u8());
                    most_significant_bit = Some(buffer.get_u8());
                }
                opcode => unimplemented!("unknown varbit_config opcode {}", opcode),
            }
        }
    }
}

/// Save the varbit configs as `varbit_configs.json`. Exposed as `--dump varbit_configs`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output)?;
    let mut vb_configs = VarbitConfig::dump_all(config)?.into_values().collect::<Vec<_>>();
    vb_configs.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(path!(config.output / "varbit_configs.json"))?;
    let data = serde_json::to_string_pretty(&vb_configs).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
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
