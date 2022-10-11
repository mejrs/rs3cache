use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};

use bytes::{Buf, Bytes};
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
    cache::{buf::BufExtra, error::CacheResult, index::CacheIndex},
    definitions::indextype::{ConfigType, IndexType},
};
/// Describes (part of) ground colour.
#[cfg_attr(feature = "pyo3", pyclass(frozen))]
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct Overlay {
    /// Id of the [`Overlay`] configuration.
    pub id: u32,
    /// Primary colour of the [`Overlay`] configuration.
    pub primary_colour: Option<[u8; 3]>,

    #[cfg(feature = "osrs")]
    pub texture: Option<u8>,

    #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
    op_3: Option<u16>,

    op_5: Option<bool>,
    /// Secondary colour of the [`Overlay`] configuration.
    pub secondary_colour: Option<[u8; 3]>,

    #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
    op_8: Option<bool>,

    #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
    op_9: Option<u16>,

    #[cfg(feature = "rs3")]
    op_10: Option<bool>,

    #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
    op_11: Option<u8>,

    #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
    op_12: Option<bool>,

    #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
    ternary_colour: Option<[u8; 3]>,

    #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
    op_14: Option<u8>,

    #[cfg(feature = "2009_1_shim")]
    op_15: Option<u16>,

    #[cfg(any(feature = "rs3", feature = "2010_1_shim"))]
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

    fn deserialize(id: u32, mut buffer: Bytes) -> Overlay {
        let mut overlay = Overlay { id, ..Default::default() };

        loop {
            let opcode = buffer.get_u8();
            match opcode {
                0 => {
                    assert!(!buffer.has_remaining());
                    break overlay;
                }
                1 => overlay.primary_colour = Some(buffer.get_rgb()),
                #[cfg(feature = "osrs")]
                2 => overlay.texture = Some(buffer.get_u8()),
                #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
                3 => overlay.op_3 = Some(buffer.get_u16()),
                5 => overlay.op_5 = Some(true),
                7 => overlay.secondary_colour = Some(buffer.get_rgb()),
                #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
                8 => overlay.op_8 = Some(true),
                #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
                9 => overlay.op_9 = Some(buffer.get_u16()),
                #[cfg(feature = "rs3")]
                10 => overlay.op_10 = Some(true),
                #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
                11 => overlay.op_11 = Some(buffer.get_u8()),
                #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
                12 => overlay.op_12 = Some(true),
                #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
                13 => overlay.ternary_colour = Some(buffer.get_rgb()),
                #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
                14 => overlay.op_14 = Some(buffer.get_u8()),
                #[cfg(feature = "2009_1_shim")]
                15 => overlay.op_15 = Some(buffer.get_u16()),
                #[cfg(any(feature = "rs3", feature = "2010_1_shim"))]
                16 => overlay.op_16 = Some(buffer.get_u8()),
                missing => unimplemented!("Overlay::deserialize cannot deserialize opcode {} in id {}: {:?}", missing, id, buffer),
            }
        }
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for Overlay {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
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
