//! Describes the properties of enums.
#![allow(non_camel_case_types, missing_docs)]

use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    fs::{self, File},
    io::Write,
    iter,
};

use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, PyObjectProtocol};
use serde::Serialize;

use crate::{
    cache::{buf::Buffer, index::CacheIndex, indextype::IndexType},
    utils::error::CacheResult,
};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Clone, Copy)]
pub enum KeyType {
    Uninit,
    Int_0,
    Int_9,
    Int_10,
    Int_17,
    Int_22,
    Int_23,
    Int_25,
    Int_26,
    Int_30,
    Int_32,
    Int_33,
    Int_39,
    Int_41,
    Int_42,
    Int_73,
    Int_74,
    Int_105,
    Int_115,
    Int_126,
    Int_128,
}

impl TryFrom<u8> for KeyType {
    type Error = String;
    fn try_from(discriminant: u8) -> Result<Self, Self::Error> {
        match discriminant {
            0 => Ok(Self::Int_0),
            9 => Ok(Self::Int_9),
            10 => Ok(Self::Int_10),
            17 => Ok(Self::Int_17),
            22 => Ok(Self::Int_22),
            23 => Ok(Self::Int_23),
            25 => Ok(Self::Int_25),
            26 => Ok(Self::Int_26),
            30 => Ok(Self::Int_30),
            32 => Ok(Self::Int_32),
            33 => Ok(Self::Int_33),
            39 => Ok(Self::Int_39),
            41 => Ok(Self::Int_41),
            42 => Ok(Self::Int_42),
            73 => Ok(Self::Int_73),
            74 => Ok(Self::Int_74),
            105 => Ok(Self::Int_105),
            126 => Ok(Self::Int_126),
            128 => Ok(Self::Int_128),
            other => Err(format!("Unknown keytype discriminant {}", other)),
        }
    }
}

impl KeyType {
    pub fn is_init(&self) -> bool {
        *self == Self::Uninit
    }
}

impl Default for KeyType {
    fn default() -> Self {
        Self::Uninit
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Clone, Copy)]
pub enum ValueType {
    Uninit,
    Int_0,
    Int_1,
    Int_3,
    Int_6,
    Int_8,
    Int_9,
    Int_10,
    Int_11,
    Int_17,
    Int_21,
    COORDINATE,
    Int_23,
    Int_24,
    Int_25,
    Int_26,
    Int_28,
    Int_29,
    Int_30,
    Int_31,
    Int_32,
    Int_33,
    STRING_36,
    Int_37,
    Int_39,
    Int_41,
    Int_42,
    Int_57,
    Int_73,
    Int_74,
    Int_97,
    Int_99,
    Int_102,
    Int_105,
    Int_115,
    Int_126,
    Int_128,
}
impl TryFrom<u8> for ValueType {
    type Error = String;
    fn try_from(discriminant: u8) -> Result<Self, Self::Error> {
        match discriminant {
            0 => Ok(Self::Int_0),
            1 => Ok(Self::Int_1),
            3 => Ok(Self::Int_3),
            6 => Ok(Self::Int_6),
            8 => Ok(Self::Int_8),
            9 => Ok(Self::Int_9),
            10 => Ok(Self::Int_10),
            11 => Ok(Self::Int_11),
            17 => Ok(Self::Int_17),
            21 => Ok(Self::Int_21),
            22 => Ok(Self::COORDINATE),
            23 => Ok(Self::Int_23),
            24 => Ok(Self::Int_24),
            25 => Ok(Self::Int_25),
            26 => Ok(Self::Int_26),
            28 => Ok(Self::Int_28),
            29 => Ok(Self::Int_29),
            30 => Ok(Self::Int_30),
            31 => Ok(Self::Int_31),
            32 => Ok(Self::Int_32),
            33 => Ok(Self::Int_33),
            36 => Ok(Self::STRING_36),
            37 => Ok(Self::Int_37),
            39 => Ok(Self::Int_39),
            41 => Ok(Self::Int_41),
            42 => Ok(Self::Int_42),
            57 => Ok(Self::Int_57),
            73 => Ok(Self::Int_73),
            74 => Ok(Self::Int_74),
            97 => Ok(Self::Int_97),
            99 => Ok(Self::Int_99),
            102 => Ok(Self::Int_102),
            105 => Ok(Self::Int_105),
            115 => Ok(Self::Int_115),
            126 => Ok(Self::Int_126),
            128 => Ok(Self::Int_128),
            other => Err(format!("Unknown valuetype discriminant {}", other)),
        }
    }
}

impl ValueType {
    pub fn is_init(&self) -> bool {
        *self == Self::Uninit
    }
}

impl Default for ValueType {
    fn default() -> Self {
        Self::Uninit
    }
}

#[derive(Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(untagged)]
pub enum Value {
    /// The integer variant.
    Integer(i32),
    /// The string variant.
    String(String),
}

#[cfg(feature = "pyo3")]
impl IntoPy<Py<PyAny>> for Value {
    fn into_py(self, py: Python) -> Py<PyAny> {
        match self {
            Self::Integer(val) => val.into_py(py),
            Self::String(val) => val.into_py(py),
        }
   }
}

/// Describes the properties of a given enum.
#[cfg_eval]
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", macro_utils::pyo3_get_all)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct Enum {
    /// Its id.
    pub id: u32,
    pub unknown_131: Option<bool>,
    #[serde(skip_serializing_if = "KeyType::is_init")]
    key_type: KeyType,
    #[serde(skip_serializing_if = "ValueType::is_init")]
    value_type: ValueType,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub variants: BTreeMap<i32, Value>,
    pub default: Option<Value>,
}

impl Enum {
    /// Returns a mapping of all [`Enum`]s.
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::ENUM_CONFIG, config)?.into_iter();

        let enums = archives
            .flat_map(|archive| {
                let archive_id = archive.archive_id();
                archive
                    .take_files()
                    .into_iter()
                    .map(move |(file_id, file)| (archive_id << 8 | file_id, file))
            })
            .map(|(id, file)| (id, Self::deserialize(id, file)))
            .collect::<BTreeMap<u32, Self>>();
        Ok(enums)
    }

    fn deserialize(id: u32, file: Vec<u8>) -> Self {
        let mut buffer = Buffer::new(file);
        let mut r#enum = Self { id, ..Default::default() };

        loop {
            match buffer.read_unsigned_byte() {
                0 => {
                    debug_assert_eq!(buffer.remaining(), 0);

                    break r#enum;
                }
                1 => r#enum.key_type = buffer.read_unsigned_byte().try_into().unwrap(),
                2 => r#enum.value_type = buffer.read_unsigned_byte().try_into().unwrap(),
                101 => r#enum.key_type = buffer.read_unsigned_byte().try_into().unwrap(),
                102 => r#enum.value_type = buffer.read_unsigned_byte().try_into().unwrap(),
                3 => r#enum.default = Some(Value::String(buffer.read_string())),
                4 => r#enum.default = Some(Value::Integer(buffer.read_int())),
                5 => {
                    let count = buffer.read_unsigned_short() as usize;
                    r#enum.variants = iter::repeat_with(|| (buffer.read_int(), Value::String(buffer.read_string())))
                        .take(count)
                        .collect();
                }
                6 => {
                    let count = buffer.read_unsigned_short() as usize;
                    r#enum.variants = iter::repeat_with(|| (buffer.read_int(), Value::Integer(buffer.read_int())))
                        .take(count)
                        .collect();
                }
                7 => {
                    let _max = buffer.read_unsigned_short();
                    let count = buffer.read_unsigned_short() as usize;
                    r#enum.variants = iter::repeat_with(|| (buffer.read_unsigned_short() as i32, Value::String(buffer.read_string())))
                        .take(count)
                        .collect();
                }
                8 => {
                    let _max = buffer.read_unsigned_short();
                    let count = buffer.read_unsigned_short() as usize;
                    r#enum.variants = iter::repeat_with(|| (buffer.read_unsigned_short() as i32, Value::Integer(buffer.read_int())))
                        .take(count)
                        .collect();
                }
                131 => r#enum.unknown_131 = Some(true),

                missing => unimplemented!("Enum::deserialize cannot deserialize opcode {} in id {}", missing, id),
            }
        }
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for Enum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

#[cfg(feature = "pyo3")]
#[pyproto]
impl PyObjectProtocol for Enum {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Enum({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Enum({})", serde_json::to_string(self).unwrap()))
    }
}

/// Save the item configs as `enums.json`. Exposed as `--dump enums`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output)?;
    let mut enums = Enum::dump_all(config)?.into_values().collect::<Vec<_>>();
    enums.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(path!(config.output / "enums.json"))?;
    let data = serde_json::to_string_pretty(&enums).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}
