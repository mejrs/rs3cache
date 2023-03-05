//! Describes the properties of enums.

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    iter,
};

use ::error::Context;
use bytes::{Buf, Bytes};
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rs3cache_backend::{
    buf::{BufExtra, JString},
    error::{self, CacheResult},
    index::CacheIndex,
};
use serde::Serialize;

use crate::definitions::indextype::IndexType;

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Clone, Copy)]
pub enum KeyType {
    Uninit = -1,
    Int0 = 0,
    Int9 = 9,
    Int10 = 10,
    Int17 = 17,
    Int22 = 22,
    Int23 = 23,
    Int25 = 25,
    Int26 = 26,
    Int30 = 30,
    Int32 = 32,
    Int33 = 33,
    Int39 = 39,
    Int41 = 41,
    Int42 = 42,
    Int73 = 73,
    Int74 = 74,
    Int105 = 105,
    Int115 = 115,
    Int126 = 126,
    Int128 = 128,
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
            0 => Ok(Self::Int0),
            9 => Ok(Self::Int9),
            10 => Ok(Self::Int10),
            17 => Ok(Self::Int17),
            22 => Ok(Self::Int22),
            23 => Ok(Self::Int23),
            25 => Ok(Self::Int25),
            26 => Ok(Self::Int26),
            30 => Ok(Self::Int30),
            32 => Ok(Self::Int32),
            33 => Ok(Self::Int33),
            39 => Ok(Self::Int39),
            41 => Ok(Self::Int41),
            42 => Ok(Self::Int42),
            73 => Ok(Self::Int73),
            74 => Ok(Self::Int74),
            105 => Ok(Self::Int105),
            126 => Ok(Self::Int126),
            128 => Ok(Self::Int128),
            other => Err(format!("Unknown keytype discriminant {other}")),
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
    Int0 = 0,
    Int1 = 1,
    Int3 = 3,
    Int6 = 6,
    Int8 = 8,
    Int9 = 9,
    Int10 = 10,
    Int11 = 11,
    Int17 = 17,
    Int21 = 21,
    COORDINATE = 22,
    Int23 = 23,
    Int24 = 24,
    Int25 = 25,
    Int26 = 26,
    Int28 = 28,
    Int29 = 29,
    Int30 = 30,
    Int31 = 31,
    Int32 = 32,
    Int33 = 33,
    STRING36 = 36,
    Int37 = 37,
    Int39 = 39,
    Int41 = 41,
    Int42 = 42,
    Int57 = 57,
    Int73 = 73,
    Int74 = 74,
    Int97 = 97,
    Int99 = 99,
    Int102 = 102,
    Int105 = 105,
    Int115 = 115,
    Int126 = 126,
    Int128 = 128,
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
            0 => Ok(Self::Int0),
            1 => Ok(Self::Int1),
            3 => Ok(Self::Int3),
            6 => Ok(Self::Int6),
            8 => Ok(Self::Int8),
            9 => Ok(Self::Int9),
            10 => Ok(Self::Int10),
            11 => Ok(Self::Int11),
            17 => Ok(Self::Int17),
            21 => Ok(Self::Int21),
            22 => Ok(Self::COORDINATE),
            23 => Ok(Self::Int23),
            24 => Ok(Self::Int24),
            25 => Ok(Self::Int25),
            26 => Ok(Self::Int26),
            28 => Ok(Self::Int28),
            29 => Ok(Self::Int29),
            30 => Ok(Self::Int30),
            31 => Ok(Self::Int31),
            32 => Ok(Self::Int32),
            33 => Ok(Self::Int33),
            36 => Ok(Self::STRING36),
            37 => Ok(Self::Int37),
            39 => Ok(Self::Int39),
            41 => Ok(Self::Int41),
            42 => Ok(Self::Int42),
            57 => Ok(Self::Int57),
            73 => Ok(Self::Int73),
            74 => Ok(Self::Int74),
            97 => Ok(Self::Int97),
            99 => Ok(Self::Int99),
            102 => Ok(Self::Int102),
            105 => Ok(Self::Int105),
            115 => Ok(Self::Int115),
            126 => Ok(Self::Int126),
            128 => Ok(Self::Int128),
            other => Err(format!("Unknown valuetype discriminant {other}")),
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
    String(JString<Bytes>),
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

#[cfg_attr(feature = "pyo3", pyclass(frozen, get_all))]
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
        let archives = CacheIndex::new(IndexType::ENUM_CONFIG, config.input.clone())?.into_iter();

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
    fs::create_dir_all(&config.output).with_context(|| error::Io { path: config.output.clone() })?;
    let mut enums = Enum::dump_all(config)?.into_values().collect::<Vec<_>>();
    enums.sort_unstable_by_key(|loc| loc.id);
    let path = path!(config.output / "enums.json");
    let mut file = File::create(&path).with_context(|| error::Io { path: path.clone() })?;

    let data = serde_json::to_string_pretty(&enums).unwrap();
    file.write_all(data.as_bytes()).context(error::Io { path })?;

    Ok(())
}
