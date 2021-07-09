//! Describes the properties of items.

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, PyObjectProtocol};
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
    cache::{buf::  Buffer, index::CacheIndex, indextype::IndexType},
    structures::paramtable::ParamTable,
    utils::error::CacheResult,
};

/// Describes the properties of a given item.
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[skip_serializing_none]
#[derive(Serialize, Debug, Default)]
pub struct ItemConfig {
    /// Its id.
    pub id: u32,
    pub base_model: Option<u32>,
    pub name: Option<String>,
    pub buff_effect: Option<String>,
    pub rotation: Option<Rotation>,
    pub translation: Option<Translation>,
    pub stackable: Option<bool>,
    pub value: Option<i32>,
    pub equipslot_id: Option<u8>,
    pub equip_id: Option<u8>,
    pub unknown_15: Option<bool>,
    pub is_members: Option<bool>,
    pub multi_stack_size: Option<u16>,
    pub male_models: Option<[Option<u32>; 3]>,
    pub female_models: Option<[Option<u32>; 3]>,
    pub unknown_27: Option<u8>,
    pub ground_actions: Option<[Option<String>; 5]>,
    pub widget_actions: Option<[Option<String>; 5]>,
    #[serde(flatten)]
    pub colour_replacements: Option<ColourReplacements>,
    #[serde(flatten)]
    pub textures: Option<Textures>,
    pub recolour_palette: Option<RecolourPalette>,
    pub name_color: Option<i32>,
    pub recolour_indices: Option<u16>,
    pub retexture_indices: Option<u16>,
    pub is_tradeable: Option<bool>,
    pub ge_buy_limit: Option<i32>,
    pub male_head_models: Option<[Option<u32>; 2]>,
    pub female_head_models: Option<[Option<u32>; 2]>,
    pub category: Option<u16>,
    pub model_yaw: Option<u16>,
    pub dummy_item: Option<u8>,
    pub note_data: Option<u16>,
    pub note_template: Option<u16>,
    pub stack_info: Option<[Option<(u16, u16)>; 10]>,
    pub scale: Option<[Option<u16>; 3]>,
    pub contrast: Option<i8>,
    pub team: Option<u8>,
    pub ambiance: Option<i8>,
    pub loan_id: Option<u16>,
    pub loan_template: Option<u16>,
    pub male_translate: Option<u32>,
    pub female_translate: Option<u32>,
    pub quests: Option<Quests>,
    pub pick_size_shift: Option<u8>,
    pub unknown_bind_link: Option<u16>,
    pub bind_template: Option<u16>,
    pub ground_action_cursor: Option<[Option<u16>; 5]>,
    pub widget_action_cursor: Option<[Option<u16>; 5]>,
    pub dummy: Option<bool>,
    pub randomize_ground_pos: Option<bool>,
    pub combine_info: Option<u16>,
    pub combine_template: Option<u16>,
    pub combine_num_required: Option<u16>,
    pub combine_shard_name: Option<String>,
    pub never_stackable: Option<bool>,
    pub unknown_167: Option<bool>,
    pub unknown_168: Option<bool>,
    #[serde(flatten)]
    pub params: Option<ParamTable>,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl ItemConfig {
    #[getter]
    fn id(&self) -> PyResult<u32> {
        Ok(self.id)
    }
    #[getter]
    fn base_model(&self) -> PyResult<Option<u32>> {
        Ok(self.base_model)
    }
    #[getter]
    fn name(&self) -> PyResult<Option<String>> {
        Ok(self.name.clone())
    }
    #[getter]
    fn buff_effect(&self) -> PyResult<Option<String>> {
        Ok(self.buff_effect.clone())
    }
    #[getter]
    fn rotation(&self) -> PyResult<Option<Rotation>> {
        Ok(self.rotation)
    }
    #[getter]
    fn translation(&self) -> PyResult<Option<Translation>> {
        Ok(self.translation)
    }
    #[getter]
    fn stackable(&self) -> PyResult<Option<bool>> {
        Ok(self.stackable)
    }
    #[getter]
    fn value(&self) -> PyResult<Option<i32>> {
        Ok(self.value)
    }
    #[getter]
    fn equipslot_id(&self) -> PyResult<Option<u8>> {
        Ok(self.equipslot_id)
    }
    #[getter]
    fn equip_id(&self) -> PyResult<Option<u8>> {
        Ok(self.equip_id)
    }
    #[getter]
    fn unknown_15(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_15)
    }
    #[getter]
    fn is_members(&self) -> PyResult<Option<bool>> {
        Ok(self.is_members)
    }
    #[getter]
    fn multi_stack_size(&self) -> PyResult<Option<u16>> {
        Ok(self.multi_stack_size)
    }
    #[getter]
    fn male_models(&self) -> PyResult<Option<[Option<u32>; 3]>> {
        Ok(self.male_models)
    }
    #[getter]
    fn female_models(&self) -> PyResult<Option<[Option<u32>; 3]>> {
        Ok(self.female_models)
    }
    #[getter]
    fn unknown_27(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_27)
    }
    #[getter]
    fn ground_actions(&self) -> PyResult<Option<[Option<String>; 5]>> {
        Ok(self.ground_actions.clone())
    }
    #[getter]
    fn widget_actions(&self) -> PyResult<Option<[Option<String>; 5]>> {
        Ok(self.widget_actions.clone())
    }
    #[getter]
    fn colour_replacements(&self) -> PyResult<Option<ColourReplacements>> {
        Ok(self.colour_replacements.clone())
    }
    #[getter]
    fn textures(&self) -> PyResult<Option<Textures>> {
        Ok(self.textures.clone())
    }
    #[getter]
    fn recolour_palette(&self) -> PyResult<Option<RecolourPalette>> {
        Ok(self.recolour_palette.clone())
    }
    #[getter]
    fn name_color(&self) -> PyResult<Option<i32>> {
        Ok(self.name_color)
    }
    #[getter]
    fn recolour_indices(&self) -> PyResult<Option<u16>> {
        Ok(self.recolour_indices)
    }
    #[getter]
    fn retexture_indices(&self) -> PyResult<Option<u16>> {
        Ok(self.retexture_indices)
    }
    #[getter]
    fn is_tradeable(&self) -> PyResult<Option<bool>> {
        Ok(self.is_tradeable)
    }
    #[getter]
    fn ge_buy_limit(&self) -> PyResult<Option<i32>> {
        Ok(self.ge_buy_limit)
    }
    #[getter]
    fn male_head_models(&self) -> PyResult<Option<[Option<u32>; 2]>> {
        Ok(self.male_head_models)
    }
    #[getter]
    fn female_head_models(&self) -> PyResult<Option<[Option<u32>; 2]>> {
        Ok(self.female_head_models)
    }
    #[getter]
    fn category(&self) -> PyResult<Option<u16>> {
        Ok(self.category)
    }
    #[getter]
    fn model_yaw(&self) -> PyResult<Option<u16>> {
        Ok(self.model_yaw)
    }
    #[getter]
    fn dummy_item(&self) -> PyResult<Option<u8>> {
        Ok(self.dummy_item)
    }
    #[getter]
    fn note_data(&self) -> PyResult<Option<u16>> {
        Ok(self.note_data)
    }
    #[getter]
    fn note_template(&self) -> PyResult<Option<u16>> {
        Ok(self.note_template)
    }
    #[getter]
    fn stack_info(&self) -> PyResult<Option<[Option<(u16, u16)>; 10]>> {
        Ok(self.stack_info)
    }
    #[getter]
    fn scale(&self) -> PyResult<Option<[Option<u16>; 3]>> {
        Ok(self.scale)
    }
    #[getter]
    fn contrast(&self) -> PyResult<Option<i8>> {
        Ok(self.contrast)
    }
    #[getter]
    fn team(&self) -> PyResult<Option<u8>> {
        Ok(self.team)
    }
    #[getter]
    fn ambiance(&self) -> PyResult<Option<i8>> {
        Ok(self.ambiance)
    }
    #[getter]
    fn loan_id(&self) -> PyResult<Option<u16>> {
        Ok(self.loan_id)
    }
    #[getter]
    fn loan_template(&self) -> PyResult<Option<u16>> {
        Ok(self.loan_template)
    }
    #[getter]
    fn male_translate(&self) -> PyResult<Option<u32>> {
        Ok(self.male_translate)
    }
    #[getter]
    fn female_translate(&self) -> PyResult<Option<u32>> {
        Ok(self.female_translate)
    }
    #[getter]
    fn quests(&self) -> PyResult<Option<Quests>> {
        Ok(self.quests.clone())
    }
    #[getter]
    fn pick_size_shift(&self) -> PyResult<Option<u8>> {
        Ok(self.pick_size_shift)
    }
    #[getter]
    fn unknown_bind_link(&self) -> PyResult<Option<u16>> {
        Ok(self.unknown_bind_link)
    }
    #[getter]
    fn bind_template(&self) -> PyResult<Option<u16>> {
        Ok(self.bind_template)
    }
    #[getter]
    fn ground_action_cursor(&self) -> PyResult<Option<[Option<u16>; 5]>> {
        Ok(self.ground_action_cursor)
    }
    #[getter]
    fn widget_action_cursor(&self) -> PyResult<Option<[Option<u16>; 5]>> {
        Ok(self.widget_action_cursor)
    }
    #[getter]
    fn dummy(&self) -> PyResult<Option<bool>> {
        Ok(self.dummy)
    }
    #[getter]
    fn randomize_ground_pos(&self) -> PyResult<Option<bool>> {
        Ok(self.randomize_ground_pos)
    }
    #[getter]
    fn combine_info(&self) -> PyResult<Option<u16>> {
        Ok(self.combine_info)
    }
    #[getter]
    fn combine_template(&self) -> PyResult<Option<u16>> {
        Ok(self.combine_template)
    }
    #[getter]
    fn combine_num_required(&self) -> PyResult<Option<u16>> {
        Ok(self.combine_num_required)
    }
    #[getter]
    fn combine_shard_name(&self) -> PyResult<Option<String>> {
        Ok(self.combine_shard_name.clone())
    }
    #[getter]
    fn never_stackable(&self) -> PyResult<Option<bool>> {
        Ok(self.never_stackable)
    }
    #[getter]
    fn unknown_167(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_167)
    }
    #[getter]
    fn unknown_168(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_168)
    }
    #[getter]
    fn params(&self) -> PyResult<Option<ParamTable>> {
        Ok(self.params.clone())
    }
}

impl ItemConfig {
    /// Returns a mapping of all [`ItemConfig`]s.
    pub fn dump_all() -> CacheResult<HashMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::OBJ_CONFIG)?.into_iter();

        let locations = archives
            .flat_map(|archive| {
                let archive_id = archive.archive_id();
                archive
                    .take_files()
                    .into_iter()
                    .map(move |(file_id, file)| (archive_id << 8 | file_id, file))
            })
            .map(|(id, file)| (id, Self::deserialize(id, file)))
            .collect::<HashMap<u32, Self>>();
        Ok(locations)
    }

    fn deserialize(id: u32, file: Vec<u8>) -> Self {
        let mut buffer =  Buffer::new(file);
        let mut item = Self { id, ..Default::default() };

        loop {
            match buffer.read_unsigned_byte() {
                0 => {
                    debug_assert_eq!(buffer.remaining(), 0);
                    break item;
                }
                1 => item.base_model = Some(buffer.read_smart32().unwrap()),
                2 => item.name = Some(buffer.read_string()),
                3 => item.buff_effect = Some(buffer.read_string()),
                4 => item.rotation.get_or_insert_default().yaw = buffer.read_unsigned_short(),
                5 => item.rotation.get_or_insert_default().pitch = buffer.read_unsigned_short(),
                6 => item.rotation.get_or_insert_default().roll = buffer.read_unsigned_short(),
                7 => item.translation.get_or_insert_default().x = buffer.read_unsigned_short(),
                8 => item.translation.get_or_insert_default().y = buffer.read_unsigned_short(),
                11 => item.stackable = Some(true),
                12 => item.value = Some(buffer.read_int()),
                13 => item.equipslot_id = Some(buffer.read_unsigned_byte()),
                14 => item.equip_id = Some(buffer.read_unsigned_byte()),
                15 => item.unknown_15 = Some(true),
                16 => item.is_members = Some(true),
                23 => item.male_models.get_or_insert_default()[0] = Some(buffer.read_smart32().unwrap()),
                24 => item.male_models.get_or_insert_default()[1] = Some(buffer.read_smart32().unwrap()),
                25 => item.female_models.get_or_insert_default()[0] = Some(buffer.read_smart32().unwrap()),
                26 => item.female_models.get_or_insert_default()[1] = Some(buffer.read_smart32().unwrap()),
                27 => item.unknown_27 = Some(buffer.read_unsigned_byte()),
                opcode @ 30..=34 => {
                    item.ground_actions.get_or_insert([None, None, None, None, None])[opcode as usize - 30] = Some(buffer.read_string())
                }
                opcode @ 35..=39 => {
                    item.widget_actions.get_or_insert([None, None, None, None, None])[opcode as usize - 35] = Some(buffer.read_string())
                }
                40 => item.colour_replacements = Some(ColourReplacements::deserialize(&mut buffer)),
                41 => item.textures = Some(Textures::deserialize(&mut buffer)),
                42 => item.recolour_palette = Some(RecolourPalette::deserialize(&mut buffer)),
                44 => item.recolour_indices = Some(buffer.read_masked_index()),
                45 => item.retexture_indices = Some(buffer.read_masked_index()),
                65 => item.is_tradeable = Some(true),
                69 => item.ge_buy_limit = Some(buffer.read_int()),
                78 => item.male_models.get_or_insert_default()[2] = Some(buffer.read_smart32().unwrap()),
                79 => item.female_models.get_or_insert_default()[2] = Some(buffer.read_smart32().unwrap()),
                90 => item.male_head_models.get_or_insert_default()[0] = Some(buffer.read_smart32().unwrap()),
                91 => item.female_head_models.get_or_insert_default()[0] = Some(buffer.read_smart32().unwrap()),
                92 => item.male_head_models.get_or_insert_default()[1] = Some(buffer.read_smart32().unwrap()),
                93 => item.female_head_models.get_or_insert_default()[1] = Some(buffer.read_smart32().unwrap()),
                94 => item.category = Some(buffer.read_unsigned_short()),
                95 => item.model_yaw = Some(buffer.read_unsigned_short()),
                96 => item.dummy_item = Some(buffer.read_unsigned_byte()),
                97 => item.note_data = Some(buffer.read_unsigned_short()),
                98 => item.note_template = Some(buffer.read_unsigned_short()),
                opcode @ 100..=109 => {
                    item.stack_info.get_or_insert_default()[opcode as usize - 100] =
                        Some((buffer.read_unsigned_short(), buffer.read_unsigned_short()))
                }
                opcode @ 110..=112 => item.scale.get_or_insert_default()[opcode as usize - 110] = Some(buffer.read_unsigned_short()),
                113 => item.ambiance = Some(buffer.read_byte()),
                114 => item.contrast = Some(buffer.read_byte()),
                115 => item.team = Some(buffer.read_unsigned_byte()),
                121 => item.loan_id = Some(buffer.read_unsigned_short()),
                122 => item.loan_template = Some(buffer.read_unsigned_short()),
                125 => item.male_translate = Some(buffer.read_3_unsigned_bytes()),
                126 => item.female_translate = Some(buffer.read_3_unsigned_bytes()),
                132 => item.quests = Some(Quests::deserialize(&mut buffer)),
                134 => item.pick_size_shift = Some(buffer.read_unsigned_byte()),
                139 => item.unknown_bind_link = Some(buffer.read_unsigned_short()),
                140 => item.bind_template = Some(buffer.read_unsigned_short()),
                opcode @ 142..=146 => item.ground_action_cursor.get_or_insert_default()[opcode as usize - 142] = Some(buffer.read_unsigned_short()),
                opcode @ 150..=154 => item.widget_action_cursor.get_or_insert_default()[opcode as usize - 150] = Some(buffer.read_unsigned_short()),
                157 => item.randomize_ground_pos = Some(true),
                161 => item.combine_info = Some(buffer.read_unsigned_short()),
                162 => item.combine_template = Some(buffer.read_unsigned_short()),
                163 => item.combine_num_required = Some(buffer.read_unsigned_short()),
                164 => item.combine_shard_name = Some(buffer.read_string()),
                165 => item.never_stackable = Some(true),
                167 => item.unknown_167 = Some(true),
                168 => item.unknown_168 = Some(true),
                249 => item.params = Some(ParamTable::deserialize(&mut buffer)),

                missing => unimplemented!("ItemConfig::deserialize cannot deserialize opcode {} in id {}", missing, id),
            }
        }
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for ItemConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

#[cfg(feature = "pyo3")]
#[pyproto]
impl PyObjectProtocol for ItemConfig {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("ItemConfig({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("ItemConfig({})", serde_json::to_string(self).unwrap()))
    }
}

/// Defines the structs used as fields of [`ItemConfig`],
pub mod item_config_fields {
    #![allow(missing_docs)]
    use std::{collections::HashMap, iter};

    #[cfg(feature = "pyo3")]
    use pyo3::prelude::*;
    use serde::Serialize;

    use crate::cache::buf::  Buffer;

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(PartialEq, Eq, Serialize, Debug, Default, Clone, Copy)]
    pub struct Rotation {
        pub yaw: u16,
        pub pitch: u16,
        pub roll: u16,
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(PartialEq, Eq, Serialize, Debug, Default, Clone, Copy)]
    pub struct Translation {
        pub x: u16,
        pub y: u16,
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct ColourReplacements {
        pub colours: Vec<(u16, u16)>,
    }

    #[cfg(feature = "pyo3")]
    #[pymethods]
    impl ColourReplacements {
        #[getter]
        fn colours(&self) -> PyResult<Vec<(u16, u16)>> {
            Ok(self.colours.clone())
        }
    }

    impl ColourReplacements {
        pub fn deserialize(buffer: &mut  Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_unsigned_byte() as usize;
            let colours = iter::repeat_with(|| (buffer.read_unsigned_short(), buffer.read_unsigned_short()))
                .take(count)
                .collect::<Vec<_>>();
            Self { colours }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct Textures {
        pub textures: HashMap<u16, u16>,
    }

    #[cfg(feature = "pyo3")]
    #[pymethods]
    impl Textures {
        #[getter]
        fn textures(&self) -> PyResult<HashMap<u16, u16>> {
            Ok(self.textures.clone())
        }
    }

    impl Textures {
        pub fn deserialize(buffer: &mut  Buffer<Vec<u8>>) -> Textures {
            let count = buffer.read_unsigned_byte() as usize;
            let textures = iter::repeat_with(|| (buffer.read_unsigned_short(), buffer.read_unsigned_short()))
                .take(count)
                .collect::<HashMap<_, _>>();
            Textures { textures }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Debug, Serialize, Clone)]
    pub struct Quests {
        pub quests: Vec<u16>,
    }

    impl Quests {
        pub fn deserialize(buffer: &mut  Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_unsigned_byte() as usize;
            let quests = iter::repeat_with(|| buffer.read_unsigned_short()).take(count).collect();
            Self { quests }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Debug, Serialize, Clone, Copy)]
    pub struct StackInfo {
        unknown_1: u16,
        unknown_2: u16,
    }

    #[cfg(feature = "pyo3")]
    #[pymethods]
    impl StackInfo {
        #[getter]
        fn unknown_1(&self) -> PyResult<u16> {
            Ok(self.unknown_1)
        }
        #[getter]
        fn unknown_2(&self) -> PyResult<u16> {
            Ok(self.unknown_2)
        }
    }

    impl StackInfo {
        pub fn deserialize(buffer: &mut  Buffer<Vec<u8>>) -> Self {
            let unknown_1 = buffer.read_unsigned_short();
            let unknown_2 = buffer.read_unsigned_short();
            Self { unknown_1, unknown_2 }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)] 
    #[derive(Debug, Serialize, Clone)]
    pub struct RecolourPalette {
        pub palette: Vec<i8>,
    }

    #[cfg(feature = "pyo3")]
    #[pymethods]
    impl RecolourPalette {
        #[getter]
        fn palette(&self) -> PyResult<Vec<i8>> {
            Ok(self.palette.clone())
        }
    }

    impl RecolourPalette {
        pub fn deserialize(buffer: &mut  Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_unsigned_byte() as usize;

            let palette = iter::repeat_with(|| buffer.read_byte()).take(count).collect::<Vec<_>>();
            Self { palette }
        }
    }
}

use item_config_fields::*;

/// Save the item configs as `item_configs.json`. Exposed as `--dump item_configs`.
pub fn export() -> CacheResult<()> {
    fs::create_dir_all("out")?;
    let mut item_configs = ItemConfig::dump_all()?.into_values().collect::<Vec<_>>();
    item_configs.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create("out/item_configs.json")?;
    let data = serde_json::to_string_pretty(&item_configs).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}
