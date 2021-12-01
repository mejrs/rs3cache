//! Describes the properties of enums.
#![allow(non_camel_case_types, missing_docs)]

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    iter,
};

use bytes::{Buf, Bytes};
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, PyObjectProtocol};
use serde::Serialize;

use crate::cache::{buf::BufExtra, error::CacheResult, index::CacheIndex, indextype::IndexType};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Clone, Copy)]
pub enum KeyType {
    Uninit = -1,
    Int_0 = 0,
    Int_9 = 9,
    Int_10 = 10,
    Int_17 = 17,
    Int_22 = 22,
    Int_23 = 23,
    Int_25 = 25,
    Int_26 = 26,
    Int_30 = 30,
    Int_32 = 32,
    Int_33 = 33,
    Int_39 = 39,
    Int_41 = 41,
    Int_42 = 42,
    Int_73 = 73,
    Int_74 = 74,
    Int_105 = 105,
    Int_115 = 115,
    Int_126 = 126,
    Int_128 = 128,
}

#[cfg(feature = "pyo3")]
impl IntoPy<PyObject> for KeyType {
    fn into_py(self, py: Python) -> PyObject {
        (self as i32).into_py(py)
    }
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
    Uninit = -1,
    Int_0 = 0,
    Int_1 = 1,
    Int_3 = 3,
    Int_6 = 6,
    Int_8 = 8,
    Int_9 = 9,
    Int_10 = 10,
    Int_11 = 11,
    Int_17 = 17,
    Int_21 = 21,
    COORDINATE = 22,
    Int_23 = 23,
    Int_24 = 24,
    Int_25 = 25,
    Int_26 = 26,
    Int_28 = 28,
    Int_29 = 29,
    Int_30 = 30,
    Int_31 = 31,
    Int_32 = 32,
    Int_33 = 33,
    STRING_36 = 36,
    Int_37 = 37,
    Int_39 = 39,
    Int_41 = 41,
    Int_42 = 42,
    Int_57 = 57,
    Int_73 = 73,
    Int_74 = 74,
    Int_97 = 97,
    Int_99 = 99,
    Int_102 = 102,
    Int_105 = 105,
    Int_115 = 115,
    Int_126 = 126,
    Int_128 = 128,
}

#[cfg(feature = "pyo3")]
impl IntoPy<PyObject> for ValueType {
    fn into_py(self, py: Python) -> PyObject {
        (self as i32).into_py(py)
    }
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
#[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct Enum {
    /// Its id.
    pub id: u32,
    pub unknown_131: Option<bool>,
    #[serde(skip_serializing_if = "KeyType::is_init")]
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    key_type: KeyType,
    #[serde(skip_serializing_if = "ValueType::is_init")]
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    value_type: ValueType,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[cfg_attr(feature = "pyo3", pyo3(get))]
    pub variants: BTreeMap<i32, Value>,
    pub default: Option<Value>,
}

impl Enum {
    /// Returns a mapping of all [`Enum`]s.
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::ENUM_CONFIG, &config.input)?.into_iter();

        let enums = archives
            .map(Result::unwrap)
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

    pub fn deserialize(id: u32, mut buffer: Bytes) -> Self {
        let mut r#enum = Self { id, ..Default::default() };

        loop {
            match buffer.get_u8() {
                0 => {
                    debug_assert!(!buffer.has_remaining());

                    break r#enum;
                }
                1 => r#enum.key_type = buffer.get_u8().try_into().unwrap(),
                2 => r#enum.value_type = buffer.get_u8().try_into().unwrap(),
                101 => r#enum.key_type = buffer.get_u8().try_into().unwrap(),
                102 => r#enum.value_type = buffer.get_u8().try_into().unwrap(),
                3 => r#enum.default = Some(Value::String(buffer.get_string())),
                4 => r#enum.default = Some(Value::Integer(buffer.get_i32())),
                5 => {
                    let count = buffer.get_u16() as usize;
                    r#enum.variants = iter::repeat_with(|| (buffer.get_i32(), Value::String(buffer.get_string())))
                        .take(count)
                        .collect();
                }
                6 => {
                    let count = buffer.get_u16() as usize;
                    r#enum.variants = iter::repeat_with(|| (buffer.get_i32(), Value::Integer(buffer.get_i32())))
                        .take(count)
                        .collect();
                }
                7 => {
                    let _max = buffer.get_u16();
                    let count = buffer.get_u16() as usize;
                    r#enum.variants = iter::repeat_with(|| (buffer.get_u16() as i32, Value::String(buffer.get_string())))
                        .take(count)
                        .collect();
                }
                8 => {
                    let _max = buffer.get_u16();
                    let count = buffer.get_u16() as usize;
                    r#enum.variants = iter::repeat_with(|| (buffer.get_u16() as i32, Value::Integer(buffer.get_i32())))
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
#[pymethods]
impl Enum {
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
