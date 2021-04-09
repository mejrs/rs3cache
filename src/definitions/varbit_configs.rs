//! Bitpacked player variables.
//!
//! 
//! See also [`Varp`](crate::types::variables::Varp) and [`Varbit`](crate::types::variables::Varbit).

use crate::{
    cache::{
        buf::Buffer,
        index::CacheIndex,
        indextype::{ConfigType, IndexType},
    },
    utils::error::CacheResult,
};

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

use pyo3::prelude::*;
use serde::Serialize;

/// A varbit configuration.
///
/// The 
#[pyclass]
#[allow(missing_docs)]
#[derive(Debug, Serialize)]
pub struct VarbitConfig {
    /// Id of the [`Varbit`](crate::types::variables::Varbit).
    #[pyo3(get)]
    pub id: u32,

    #[pyo3(get)]
    pub unknown_1: u8,

    /// The Varp that this varbit maps to.
    #[pyo3(get)]
    pub index: u16,

    #[pyo3(get)]
    pub least_significant_bit: u8,

    #[pyo3(get)]
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
        let mut buffer = Buffer::new(file);

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