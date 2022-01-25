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
#[cfg_attr(feature = "pyo3", pyclass)]
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct Flo {
    /// Id of the [`Flo`] configuration.
    pub id: u32,
    /// Primary colour of the [`Flo`] configuration.
    pub primary_colour: Option<[u8; 3]>,
    pub texture: Option<u8>,
    op_3: Option<bool>,
    op_5: Option<bool>,
    name: Option<String>,
    /// Secondary colour of the [`Flo`] configuration.
    pub secondary_colour: Option<[u8; 3]>,
}

impl Flo {
    /// Returns a mapping of all [`Flo`] configurations.
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Flo>> {
        let cache = CacheIndex::new(0, &config.input).unwrap();
        let archive = cache.archive(2).unwrap();
        let mut file = archive.file_named("flo.dat").unwrap();

        let count = file.try_get_u16().unwrap();
        let mut offset_data = archive.file_named("flo.idx").unwrap();

        let len = offset_data.try_get_u16().unwrap();

        let mut flos = BTreeMap::new();
        for id in 0..len {
            let piece_len = offset_data.try_get_u16().unwrap();
            let data = file.split_to(piece_len as usize);
            let flo = Flo::deserialize(id as u32, data);
            flos.insert(id as u32, flo);
        }
        Ok(flos)
    }

    fn deserialize(id: u32, mut buffer: Bytes) -> Flo {
        let mut flo = Flo { id, ..Default::default() };

        loop {
            let opcode = buffer.get_u8();
            match opcode {
                0 => {
                    assert!(!buffer.has_remaining());
                    break flo;
                }
                1 => flo.primary_colour = Some(buffer.get_rgb()),
                2 => flo.texture = Some(buffer.get_u8()),
                3 => flo.op_3 = Some(true),
                5 => flo.op_5 = Some(true),
                6 => flo.name = Some(buffer.get_string()),
                7 => flo.secondary_colour = Some(buffer.get_rgb()),
                missing => unimplemented!("Flo::deserialize cannot deserialize opcode {} in id {}", missing, id),
            }
        }
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for Flo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

///Save the maplabels as `maplabels.json`. Exposed as `--dump maplabels`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output)?;
    let mut labels = Flo::dump_all(config)?.into_values().collect::<Vec<_>>();
    labels.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(path!(&config.output / "Flos.json"))?;
    let data = serde_json::to_string_pretty(&labels)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

#[cfg(all(test, feature = "legacy"))]
mod legacy {
    use std::path::PathBuf;

    use rs3cache_backend::index::CacheIndex;

    use super::*;
    use crate::cli::Config;

    #[test]
    fn decode_flo() {
        let path = "test_data/2005_cache";
        let should_be_count = 125;

        let cache = CacheIndex::new(0, path).unwrap();
        let archive = cache.archive(2).unwrap();
        let mut file = archive.file_named("flo.dat").unwrap();

        let count = file.try_get_u16().unwrap();

        assert_eq!(count, should_be_count);

        let mut offset_data = archive.file_named("flo.idx").unwrap();

        let len = offset_data.try_get_u16().unwrap();
        for id in 0..len {
            let piece_len = offset_data.try_get_u16().unwrap();
            let data = file.split_to(piece_len as usize);
            dbg!(&data);
            let loc = Flo::deserialize(id as u32, data);
            println!("{}", loc);
        }
        assert_eq!(offset_data, &[].as_slice());
        assert_eq!(file, &[].as_slice());
    }
}
