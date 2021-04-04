use crate::{
    cache::{
        buf::Buffer,
        index::CacheIndex,
        indextype::{ConfigType, IndexType},
    },
    utils::error::CacheResult,
};
use std::collections::HashMap;

#[derive(Debug, Default, Copy, Clone)]
/// Describes the general ground colour. This colour is blended with surrounding tiles.
pub struct Underlay {
    /// Id of the underlay configuration.
    pub id: u32,
    /// Ground colour of this tile type
    pub colour: Option<(u8, u8, u8)>,
    op_2: Option<u16>,
    op_3: Option<u16>,
    op_4: Option<bool>,
    op_5: Option<bool>,
}

impl Underlay {
    /// Returns a mapping of all [`Underlay`] configurations.
    pub fn dump_all() -> CacheResult<HashMap<u32, Underlay>> {
        Ok(CacheIndex::new(IndexType::CONFIG)?
            .archive(ConfigType::UNDERLAYS)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, Underlay::deserialize(file_id, file)))
            .collect())
    }

    fn deserialize(id: u32, file: Vec<u8>) -> Underlay {
        let mut buffer = Buffer::new(file);
        let mut underlay = Underlay { id, ..Default::default() };

        loop {
            let opcode = buffer.read_unsigned_byte();
            match opcode {
                0 => {
                    assert_eq!(buffer.remaining(), 0);
                    break underlay;
                }
                1 => underlay.colour = Some(buffer.read_rgb()),
                2 => underlay.op_2 = Some(buffer.read_unsigned_short()),
                3 => underlay.op_3 = Some(buffer.read_unsigned_short()),
                4 => underlay.op_4 = Some(true),
                5 => underlay.op_5 = Some(true),

                missing => unimplemented!("Underlay::deserialize cannot deserialize opcode {} in id {}", missing, id),
            }
        }
    }
}
