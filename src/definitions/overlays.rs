use crate::cache::{
    buf::Buffer,
    index::CacheIndex,
    indextype::{ConfigType, IndexType},
};
use std::collections::HashMap;

use crate::utils::error::CacheResult;
/// Describes (part of) ground colour.
#[derive(Debug, Default)]
pub struct Overlay {
    /// Id of the [`Overlay`] configuration.
    pub id: u32,
    /// Primary colour of the [`Overlay`] configuration.
    pub primary_colour: Option<(u8, u8, u8)>,

    op_3: Option<u16>,

    op_5: Option<bool>,
    /// Secondary colour of the [`Overlay`] configuration.
    pub secondary_colour: Option<(u8, u8, u8)>,

    op_8: Option<bool>,

    op_9: Option<u16>,

    op_10: Option<bool>,

    op_11: Option<u8>,

    op_12: Option<bool>,
    ternary_colour: Option<(u8, u8, u8)>,

    op_14: Option<u8>,

    op_16: Option<u8>,
}

impl Overlay {
    /// Returns a mapping of all [`Overlay`] configurations.
    pub fn dump_all() -> CacheResult<HashMap<u32, Overlay>> {
        Ok(CacheIndex::new(IndexType::CONFIG)?
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
                3 => overlay.op_3 = Some(buffer.read_unsigned_short()),
                5 => overlay.op_5 = Some(true),
                7 => overlay.secondary_colour = Some(buffer.read_rgb()),
                8 => overlay.op_8 = Some(true),
                9 => overlay.op_9 = Some(buffer.read_unsigned_short()),
                10 => overlay.op_10 = Some(true),
                11 => overlay.op_11 = Some(buffer.read_unsigned_byte()),
                12 => overlay.op_12 = Some(true),
                13 => overlay.ternary_colour = Some(buffer.read_rgb()),
                14 => overlay.op_14 = Some(buffer.read_unsigned_byte()),
                16 => overlay.op_16 = Some(buffer.read_unsigned_byte()),
                missing => unimplemented!("Overlay::deserialize cannot deserialize opcode {} in id {}", missing, id),
            }
        }
    }
}
