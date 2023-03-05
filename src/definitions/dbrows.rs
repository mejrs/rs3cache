use core::panic::Location;
use std::collections::BTreeMap;

use ::error::Context;
use bytes::{Buf, Bytes};
use rs3cache_backend::{
    buf::{BufExtra, NotExhausted, ReadError, WithInfo},
    error::{self, CacheResult},
    index::CacheIndex,
};
use serde::Serialize;

use crate::definitions::indextype::{ConfigType, IndexType};

#[cfg_attr(feature = "pyo3", pyo3::pyclass)]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct DbRow {
    /// Its id.
    pub id: u32,
    pub unknown_1: Option<bool>,
    pub content_type: Option<u8>,
    pub data: Option<Vec<Vec<Value>>>,
}

impl DbRow {
    /// Returns a mapping of all [`DbRow`] configurations.
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, DbRow>> {
        let files = CacheIndex::new(IndexType::CONFIG, config.input.clone())?
            .archive(ConfigType::DBROWS)?
            .take_files()
            .into_iter();

        files
            .map(|(file_id, file)| try { (file_id, DbRow::deserialize(file_id, file)?) })
            .collect::<Result<BTreeMap<u32, DbRow>, ReadError>>()
            .context(error::Read { what: "dbrows" })
    }
    fn deserialize(id: u32, mut buffer: Bytes) -> Result<Self, ReadError> {
        let mut obj = Self { id, ..Default::default() };

        #[cfg(debug_assertions)]
        let mut opcodes = Vec::new();

        loop {
            let opcode = buffer.try_get_u8()?;

            #[cfg(debug_assertions)]
            opcodes.push(opcode);

            let read: Result<(), ReadError> = try {
                match opcode {
                    0 => {
                        if buffer.has_remaining() {
                            return Err(NotExhausted::new(buffer.remaining()));
                        } else {
                            break Ok(obj);
                        }
                    }
                    1 => obj.unknown_1 = Some(true),
                    3 => {
                        let size = buffer.try_get_u8()?;
                        let _objects = vec![Value::Null; size as usize];
                        loop {
                            match buffer.try_get_u8()? {
                                255 => break,
                                _index => {
                                    let amount = buffer.try_get_u8()? as usize;
                                    let types: Vec<_> = std::iter::repeat_with(|| buffer.get_smarts()).take(amount).collect();
                                    let count = buffer.get_smarts() as usize;

                                    let _subobjects = vec![Value::Null; count * amount];
                                    for _c in 0..count {
                                        for (_pos, ty) in types.iter().enumerate() {
                                            match ty {
                                                35 => {
                                                    buffer.try_get_array::<8>()?;
                                                }
                                                36 => {
                                                    buffer.try_get_string()?;
                                                }
                                                _ => {
                                                    buffer.try_get_i32()?;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    4 => obj.content_type = Some(buffer.try_get_u8()?),
                    opcode => Err(ReadError::OpcodeNotImplemented {
                        location: Location::caller(),
                        opcode,
                    })?,
                }
            };
            match read {
                Ok(()) => {
                    #[cfg(debug_assertions)]
                    opcodes.push(opcode);
                }
                Err(e) => {
                    return Err(e).map_err(Box::new).context(WithInfo {
                        #[cfg(debug_assertions)]
                        opcodes,
                        buffer,
                        #[cfg(debug_assertions)]
                        thing: obj.to_string(),
                    })
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum Value {
    Integer(i32),
    Text(String),
    Null,
}

use std::fmt::{self, Display, Formatter};

impl Display for DbRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

#[cfg(all(test, feature = "rs3"))]
mod tests {
    use super::*;
    use crate::cli::Config;

    #[test]
    fn t() -> CacheResult<()> {
        let config = Config::env();

        let _dbrows = DbRow::dump_all(&config)?;

        Ok(())
    }
}
