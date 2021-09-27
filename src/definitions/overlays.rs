use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};

use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::cache::{
    buf::Buffer,
    error::CacheResult,
    index::CacheIndex,
    indextype::{ConfigType, IndexType},
};

/// Describes (part of) ground colour.
#[cfg_attr(feature = "pyo3", pyclass)]
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct Overlay {
    /// Id of the [`Overlay`] configuration.
    pub id: u32,
    /// Primary colour of the [`Overlay`] configuration.
    pub primary_colour: Option<[u8; 3]>,

    #[cfg(feature = "osrs")]
    pub texture: Option<u8>,

    #[cfg(feature = "rs3")]
    op_3: Option<u16>,

    op_5: Option<bool>,
    /// Secondary colour of the [`Overlay`] configuration.
    pub secondary_colour: Option<[u8; 3]>,

    #[cfg(feature = "rs3")]
    op_8: Option<bool>,

    #[cfg(feature = "rs3")]
    op_9: Option<u16>,

    #[cfg(feature = "rs3")]
    op_10: Option<bool>,

    #[cfg(feature = "rs3")]
    op_11: Option<u8>,

    #[cfg(feature = "rs3")]
    op_12: Option<bool>,

    #[cfg(feature = "rs3")]
    ternary_colour: Option<[u8; 3]>,

    #[cfg(feature = "rs3")]
    op_14: Option<u8>,

    #[cfg(feature = "rs3")]
    op_16: Option<u8>,
}

impl Overlay {
    /// Returns a mapping of all [`Overlay`] configurations.
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Overlay>> {
        Ok(CacheIndex::new(IndexType::CONFIG, &config.input)?
            .archive(ConfigType::OVERLAYS)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, Overlay::deserialize(file_id, file)))
            .collect())
    }

    fn deserialize(id: u32, file: Vec<u8>) -> Overlay {
        let mut buffer = Buffer::new(file);
        let mut overlay = Overlay { id, ..Default::default() };

        loop {
            let opcode = buffer.read_unsigned_byte();
            match opcode {
                0 => {
                    assert_eq!(buffer.remaining(), 0);
                    break overlay;
                }
                1 => overlay.primary_colour = Some(buffer.read_rgb()),
                #[cfg(feature = "osrs")]
                2 => overlay.texture = Some(buffer.read_unsigned_byte()),
                #[cfg(feature = "rs3")]
                3 => overlay.op_3 = Some(buffer.read_unsigned_short()),
                5 => overlay.op_5 = Some(true),
                7 => overlay.secondary_colour = Some(buffer.read_rgb()),
                #[cfg(feature = "rs3")]
                8 => overlay.op_8 = Some(true),
                #[cfg(feature = "rs3")]
                9 => overlay.op_9 = Some(buffer.read_unsigned_short()),
                #[cfg(feature = "rs3")]
                10 => overlay.op_10 = Some(true),
                #[cfg(feature = "rs3")]
                11 => overlay.op_11 = Some(buffer.read_unsigned_byte()),
                #[cfg(feature = "rs3")]
                12 => overlay.op_12 = Some(true),
                #[cfg(feature = "rs3")]
                13 => overlay.ternary_colour = Some(buffer.read_rgb()),
                #[cfg(feature = "rs3")]
                14 => overlay.op_14 = Some(buffer.read_unsigned_byte()),
                #[cfg(feature = "rs3")]
                16 => overlay.op_16 = Some(buffer.read_unsigned_byte()),
                missing => unimplemented!("Overlay::deserialize cannot deserialize opcode {} in id {}", missing, id),
            }
        }
    }
}

///Save the maplabels as `maplabels.json`. Exposed as `--dump maplabels`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output)?;
    let mut labels = Overlay::dump_all(config)?.into_values().collect::<Vec<_>>();
    labels.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(path!(&config.output / "overlays.json"))?;
    let data = serde_json::to_string_pretty(&labels)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}
