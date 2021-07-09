#![cfg(feature = "rs3")]

use std::collections::HashMap;

use crate::{
    cache::{
        buf::  Buffer,
        index::CacheIndex,
        indextype::{ConfigType, IndexType},
    },
    utils::error::CacheResult,
};
/// A configuration of a sprite that can be drawn on the world map.
#[derive(Debug, Default, Copy, Clone)]
pub struct MapScene {
    /// Its id.
    pub id: u32,
    /// A reference to the sprite.
    pub sprite_id: Option<u32>,

    op_2: Option<u32>,

    op_3: Option<bool>,

    op_4: Option<bool>,

    op_5: Option<bool>,
}

impl MapScene {
    /// Returns a mapping of all [`MapScene`] configurations.
    pub fn dump_all() -> CacheResult<HashMap<u32, MapScene>> {
        Ok(CacheIndex::new(IndexType::CONFIG)?
            .archive(ConfigType::MAPSCENES)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, MapScene::deserialize(file_id, file)))
            .collect())
    }

   
    fn deserialize(id: u32, file: Vec<u8>) -> MapScene {
        let mut buffer =  Buffer::new(file);
        let mut mapscene = MapScene { id, ..Default::default() };

        loop {
            let opcode = buffer.read_unsigned_byte();
            match opcode {
                0 => {
                    assert_eq!(buffer.remaining(), 0);
                    break mapscene;
                }
                1 => mapscene.sprite_id = buffer.read_smart32(),
                2 => mapscene.op_2 = Some(buffer.read_3_unsigned_bytes()),
                3 => mapscene.op_3 = Some(true),
                4 => mapscene.op_4 = Some(true),
                5 => mapscene.op_5 = Some(true),
                missing => unimplemented!("MapScene::deserialize cannot deserialize opcode {} in id {}", missing, id),
            }
        }
    }
}

#[cfg(test)]
mod mapscene_tests {
    use super::*;

    #[test]
    fn check_1612() -> CacheResult<()> {
        let mapscenes = MapScene::dump_all()?;
        let has_1612 = mapscenes.values().filter_map(|mapscene| mapscene.sprite_id).any(|id| id == 1612);
        assert!(has_1612, "Missing sprite 1612");
        Ok(())
    }

    #[test]
    fn check_1609() -> CacheResult<()> {
        let mapscenes = MapScene::dump_all()?;
        let has_1609 = mapscenes.values().filter_map(|mapscene| mapscene.sprite_id).any(|id| id == 1609);
        assert!(has_1609, "Missing sprite 1612");
        Ok(())
    }
}
