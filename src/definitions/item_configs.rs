//! Describes the properties of items.

use crate::{
    cache::{buf::Buffer, index::CacheIndex, indextype::IndexType},
    structures::paramtable::ParamTable,
    utils::error::CacheResult,
};
use pyo3::{prelude::*, PyObjectProtocol};
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

/// Describes the properties of a given item.
#[allow(missing_docs)]
#[pyclass]
#[skip_serializing_none]
#[derive(Serialize, Debug, Default)]
pub struct ItemConfig {
    /// Its id.
    #[pyo3(get)]
    pub id: u32,

    #[pyo3(get)]
    pub base_model: Option<u32>,

    #[pyo3(get)]
    pub name: Option<String>,

    #[pyo3(get)]
    pub buff_effect: Option<String>,

    #[pyo3(get)]
    pub rotation: Option<Rotation>,

    #[pyo3(get)]
    pub translation: Option<Translation>,

    #[pyo3(get)]
    pub stackable: Option<bool>,

    #[pyo3(get)]
    pub value: Option<i32>,

    #[pyo3(get)]
    pub equipslot_id: Option<u8>,

    #[pyo3(get)]
    pub equip_id: Option<u8>,

    #[pyo3(get)]
    pub unknown_15: Option<bool>,

    #[pyo3(get)]
    pub is_members: Option<bool>,

    #[pyo3(get)]
    pub multi_stack_size: Option<u16>,

    #[pyo3(get)]
    pub male_models: Option<[Option<u32>; 3]>,

    #[pyo3(get)]
    pub female_models: Option<[Option<u32>; 3]>,

    #[pyo3(get)]
    pub unknown_27: Option<u8>,

    #[pyo3(get)]
    pub ground_actions: Option<[Option<String>; 5]>,

    #[pyo3(get)]
    pub widget_actions: Option<[Option<String>; 5]>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub colour_replacements: Option<ColourReplacements>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub textures: Option<Textures>,

    #[pyo3(get)]
    pub recolour_palette: Option<RecolourPalette>,

    #[pyo3(get)]
    pub name_color: Option<i32>,

    #[pyo3(get)]
    pub recolour_indices: Option<u16>,

    #[pyo3(get)]
    pub retexture_indices: Option<u16>,

    #[pyo3(get)]
    pub is_tradeable: Option<bool>,

    #[pyo3(get)]
    pub ge_buy_limit: Option<i32>,

    #[pyo3(get)]
    pub male_head_models: Option<[Option<u32>; 2]>,

    #[pyo3(get)]
    pub female_head_models: Option<[Option<u32>; 2]>,

    #[pyo3(get)]
    pub category: Option<u16>,

    #[pyo3(get)]
    pub model_yaw: Option<u16>,

    #[pyo3(get)]
    pub dummy_item: Option<u8>,

    #[pyo3(get)]
    pub note_data: Option<u16>,

    #[pyo3(get)]
    pub note_template: Option<u16>,

    #[pyo3(get)]
    pub stack_info: Option<[Option<(u16, u16)>; 10]>,

    #[pyo3(get)]
    pub scale: Option<[Option<u16>; 3]>,

    #[pyo3(get)]
    pub contrast: Option<i8>,

    #[pyo3(get)]
    pub team: Option<u8>,

    #[pyo3(get)]
    pub ambiance: Option<i8>,

    #[pyo3(get)]
    pub loan_id: Option<u16>,

    #[pyo3(get)]
    pub loan_template: Option<u16>,

    #[pyo3(get)]
    pub male_translate: Option<u32>,

    #[pyo3(get)]
    pub female_translate: Option<u32>,

    #[pyo3(get)]
    pub quests: Option<Quests>,

    #[pyo3(get)]
    pub pick_size_shift: Option<u8>,

    #[pyo3(get)]
    pub unknown_bind_link: Option<u16>,

    #[pyo3(get)]
    pub bind_template: Option<u16>,

    #[pyo3(get)]
    pub ground_action_cursor: Option<[Option<u16>; 5]>,

    #[pyo3(get)]
    pub widget_action_cursor: Option<[Option<u16>; 5]>,

    #[pyo3(get)]
    pub dummy: Option<bool>,

    #[pyo3(get)]
    pub randomize_ground_pos: Option<bool>,

    #[pyo3(get)]
    pub combine_info: Option<u16>,

    #[pyo3(get)]
    pub combine_template: Option<u16>,

    #[pyo3(get)]
    pub combine_num_required: Option<u16>,

    #[pyo3(get)]
    pub combine_shard_name: Option<String>,

    #[pyo3(get)]
    pub never_stackable: Option<bool>,

    #[pyo3(get)]
    pub unknown_167: Option<bool>,

    #[pyo3(get)]
    pub unknown_168: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub params: Option<ParamTable>,
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
        let mut buffer = Buffer::new(file);
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
    use crate::cache::buf::Buffer;
    use pyo3::prelude::*;
    use serde::Serialize;
    use std::{collections::HashMap, iter};

    #[pyclass]
    #[derive(PartialEq, Eq, Serialize, Debug, Default, Clone, Copy)]
    pub struct Rotation {
        pub yaw: u16,
        pub pitch: u16,
        pub roll: u16,
    }

    #[pyclass]
    #[derive(PartialEq, Eq, Serialize, Debug, Default, Clone, Copy)]
    pub struct Translation {
        pub x: u16,
        pub y: u16,
    }

    #[pyclass]
    #[derive(Serialize, Debug, Clone)]
    pub struct ColourReplacements {
        #[pyo3(get)]
        pub colours: Vec<(u16, u16)>,
    }

    impl ColourReplacements {
        pub fn deserialize(buffer: &mut Buffer) -> Self {
            let count = buffer.read_unsigned_byte() as usize;
            let colours = iter::repeat_with(|| (buffer.read_unsigned_short(), buffer.read_unsigned_short()))
                .take(count)
                .collect::<Vec<_>>();
            Self { colours }
        }
    }

    #[pyclass]
    #[derive(Serialize, Debug, Clone)]
    pub struct Textures {
        #[pyo3(get)]
        pub textures: HashMap<u16, u16>,
    }

    impl Textures {
        pub fn deserialize(buffer: &mut Buffer) -> Textures {
            let count = buffer.read_unsigned_byte() as usize;
            let textures = iter::repeat_with(|| (buffer.read_unsigned_short(), buffer.read_unsigned_short()))
                .take(count)
                .collect::<HashMap<_, _>>();
            Textures { textures }
        }
    }

    #[pyclass]
    #[derive(Debug, Serialize, Clone)]
    pub struct Quests {
        pub quests: Vec<u16>,
    }

    impl Quests {
        pub fn deserialize(buffer: &mut Buffer) -> Self {
            let count = buffer.read_unsigned_byte() as usize;
            let quests = iter::repeat_with(|| buffer.read_unsigned_short()).take(count).collect();
            Self { quests }
        }
    }

    #[pyclass]
    #[derive(Debug, Serialize, Clone)]
    pub struct StackInfo {
        #[pyo3(get)]
        unknown_1: u16,
        #[pyo3(get)]
        unknown_2: u16,
    }

    impl StackInfo {
        pub fn deserialize(buffer: &mut Buffer) -> Self {
            let unknown_1 = buffer.read_unsigned_short();
            let unknown_2 = buffer.read_unsigned_short();
            Self { unknown_1, unknown_2 }
        }
    }

    #[pyclass]
    #[derive(Debug, Serialize, Clone)]
    pub struct RecolourPalette {
        #[pyo3(get)]
        pub palette: Vec<i8>,
    }

    impl RecolourPalette {
        pub fn deserialize(buffer: &mut Buffer) -> Self {
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
