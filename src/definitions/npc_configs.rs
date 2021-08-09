use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};


use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, PyObjectProtocol};
use serde::Serialize;

use crate::{
    cache::{buf::Buffer, index::CacheIndex, indextype::IndexType},
    structures::paramtable::ParamTable,
    utils::error::CacheResult,
};
/// Describes the properties of a given [`Npc`](crate::definitions::npcs::Npc).
#[cfg_eval]
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", macro_utils::pyo3_get_all)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct NpcConfig {
    /// Its id.
    pub id: u32,
    #[serde(flatten)]
    pub models: Option<NpcModels>,
    pub name: Option<String>,
    pub size: Option<u8>,
    pub actions: Option<[Option<String>; 5]>,
    #[serde(flatten)]
    pub colour_replacements: Option<ColourReplacements>,
    #[serde(flatten)]
    pub texture_replacements: Option<Textures>,
    #[serde(flatten)]
    pub recolour_palette: Option<RecolourPalette>,
    pub recolour_indices: Option<u16>,
    pub retexture_indices: Option<u16>,
    #[serde(flatten)]
    pub head_models: Option<HeadModels>,
    pub draw_map_dot: Option<bool>,
    pub combat: Option<u16>,
    pub scale_xz: Option<u16>,
    pub scale_y: Option<u16>,
    pub unknown_99: Option<bool>,
    pub ambience: Option<i8>,
    pub model_contract: Option<i8>,
    pub head_icon_data: Option<Vec<(Option<u32>, Option<u32>)>>,
    pub unknown_103: Option<u16>,
    pub morphs_1: Option<NpcMorphTable>,
    pub unknown_107: Option<bool>,
    pub slow_walk: Option<bool>,
    pub animate_idle: Option<bool>,
    pub shadow: Option<Shadow>,
    pub shadow_alpha_intensity: Option<ShadowIntensity>,
    pub morphs_2: Option<ExtendedNpcMorphTable>,
    pub movement_capabilities: Option<i8>,
    #[serde(flatten)]
    pub translations: Option<Translations>,
    pub icon_height: Option<u16>,
    pub respawn_direction: Option<i8>,
    pub animation_group: Option<u16>,
    pub movement_type: Option<i8>,
    pub ambient_sound: Option<AmbientSounds>,
    pub old_cursor: Option<OldCursors>,
    pub old_cursor_2: Option<OldCursors>,
    pub attack_cursor: Option<u16>,
    pub army_icon: Option<u32>,
    pub unknown_140: Option<u8>,
    pub unknown_141: Option<bool>,
    pub mapfunction: Option<u16>,
    pub unknown_143: Option<bool>,
    pub member_actions: Option<[Option<String>; 5]>,
    pub unknown_155: Option<Unknown155>,
    pub unknown_158: Option<bool>,
    pub unknown_159: Option<bool>,
    #[serde(flatten)]
    pub quests: Option<Quests>,
    pub unknown_162: Option<bool>,
    pub unknown_163: Option<u8>,
    pub unknown_164: Option<Unknown164>,
    pub unknown_165: Option<u8>,
    pub unknown_168: Option<u8>,
    pub unknown_169: Option<bool>,
    pub action_cursors: Option<[Option<u16>; 6]>,
    pub unknown_178: Option<bool>,
    pub unknown_179: Option<Unknown179>,
    pub unknown_180: Option<u8>,
    pub unknown_182: Option<bool>,
    pub unknown_184: Option<u16>,
    #[serde(flatten)]
    pub params: Option<ParamTable>,
}

impl NpcConfig {
    /// Returns a mapping of all [npc configurations](NpcConfig)
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::NPC_CONFIG, &config)?.into_iter();

        let npc_configs = archives
            .flat_map(|archive| {
                let archive_id = archive.archive_id();
                archive
                    .take_files()
                    .into_iter()
                    .map(move |(file_id, file)| (archive_id << 7 | file_id, file))
            })
            .map(|(id, file)| (id, Self::deserialize(id, file)))
            .collect::<BTreeMap<u32, Self>>();
        Ok(npc_configs)
    }

    fn deserialize(id: u32, file: Vec<u8>) -> Self {
        let mut npc = Self { id, ..Default::default() };
        let mut buffer = Buffer::new(file);
        loop {
            match buffer.read_unsigned_byte() {
                0 => {
                    debug_assert_eq!(buffer.remaining(), 0);
                    break npc;
                }
                1 => npc.models = Some(NpcModels::deserialize(&mut buffer)),
                2 => npc.name = Some(buffer.read_string()),
                12 => npc.size = Some(buffer.read_unsigned_byte()),
                opcode @ 30..=34 => {
                    let actions = npc.actions.get_or_insert([None, None, None, None, None]);
                    actions[opcode as usize - 30] = Some(buffer.read_string());
                }
                40 => npc.colour_replacements = Some(ColourReplacements::deserialize(&mut buffer)),
                41 => npc.texture_replacements = Some(Textures::deserialize(&mut buffer)),
                42 => npc.recolour_palette = Some(RecolourPalette::deserialize(&mut buffer)),
                44 => npc.recolour_indices = Some(buffer.read_masked_index()),
                45 => npc.retexture_indices = Some(buffer.read_masked_index()),
                60 => npc.head_models = Some(HeadModels::deserialize(&mut buffer)),
                93 => npc.draw_map_dot = Some(false),
                95 => npc.combat = Some(buffer.read_unsigned_short()),
                97 => npc.scale_xz = Some(buffer.read_unsigned_short()),
                98 => npc.scale_y = Some(buffer.read_unsigned_short()),
                99 => npc.unknown_99 = Some(false),
                100 => npc.ambience = Some(buffer.read_byte()),
                101 => npc.ambience = Some(buffer.read_byte()),
                102 => npc.head_icon_data = Some(buffer.read_masked_data()),
                103 => npc.unknown_103 = Some(buffer.read_unsigned_short()),
                106 => npc.morphs_1 = Some(NpcMorphTable::deserialize(&mut buffer)),
                107 => npc.unknown_107 = Some(false),
                109 => npc.slow_walk = Some(false),
                111 => npc.animate_idle = Some(false),
                113 => npc.shadow = Some(Shadow::deserialize(&mut buffer)),
                114 => npc.shadow_alpha_intensity = Some(ShadowIntensity::deserialize(&mut buffer)),
                118 => npc.morphs_2 = Some(ExtendedNpcMorphTable::deserialize(&mut buffer)),
                119 => npc.movement_capabilities = Some(buffer.read_byte()),
                121 => npc.translations = Some(Translations::deserialize(&mut buffer)),
                123 => npc.icon_height = Some(buffer.read_unsigned_short()),
                125 => npc.respawn_direction = Some(buffer.read_byte()),
                127 => npc.animation_group = Some(buffer.read_unsigned_short()),
                128 => npc.movement_type = Some(buffer.read_byte()),
                134 => npc.ambient_sound = Some(AmbientSounds::deserialize(&mut buffer)),
                135 => npc.old_cursor = Some(OldCursors::deserialize(&mut buffer)),
                136 => npc.old_cursor_2 = Some(OldCursors::deserialize(&mut buffer)),
                137 => npc.attack_cursor = Some(buffer.read_unsigned_short()),
                138 => npc.army_icon = Some(buffer.read_smart32().unwrap()),
                140 => npc.unknown_140 = Some(buffer.read_unsigned_byte()),
                141 => npc.animate_idle = Some(true),
                142 => npc.mapfunction = Some(buffer.read_unsigned_short()),
                143 => npc.unknown_143 = Some(true),
                opcode @ 150..=154 => {
                    let actions = npc.member_actions.get_or_insert([None, None, None, None, None]);
                    actions[opcode as usize - 150] = Some(buffer.read_string());
                }
                155 => npc.unknown_155 = Some(Unknown155::deserialize(&mut buffer)),
                158 => npc.unknown_158 = Some(true),
                159 => npc.unknown_159 = Some(false),
                160 => npc.quests = Some(Quests::deserialize(&mut buffer)),
                162 => npc.unknown_162 = Some(true),
                163 => npc.unknown_163 = Some(buffer.read_unsigned_byte()),
                164 => npc.unknown_164 = Some(Unknown164::deserialize(&mut buffer)),
                165 => npc.unknown_165 = Some(buffer.read_unsigned_byte()),
                168 => npc.unknown_168 = Some(buffer.read_unsigned_byte()),
                169 => npc.unknown_169 = Some(false),
                opcode @ 170..=175 => {
                    let actions = npc.action_cursors.get_or_insert([None, None, None, None, None, None]);
                    actions[opcode as usize - 170] = Some(buffer.read_unsigned_short());
                }
                178 => npc.unknown_178 = Some(true),
                179 => npc.unknown_179 = Some(Unknown179::deserialize(&mut buffer)),
                180 => unimplemented!(),
                181 => unimplemented!(),
                182 => npc.unknown_182 = Some(true),
                184 => npc.unknown_184 = Some(buffer.read_unsigned_smart()),
                249 => npc.params = Some(ParamTable::deserialize(&mut buffer)),
                missing => {
                    unimplemented!("NpcConfig::deserialize cannot deserialize opcode {} in npc: \n {}\n", missing, npc)
                }
            }
        }
    }
}

#[cfg(feature = "pyo3")]
#[pyproto]
impl PyObjectProtocol for NpcConfig {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("NpcConfig({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("NpcConfig({})", serde_json::to_string(self).unwrap()))
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for NpcConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

/// Defines the structs used as fields of [`NpcConfig`],
pub mod npc_config_fields {
    #![allow(missing_docs)]

    use std::{collections::BTreeMap, iter};

    #[cfg(feature = "pyo3")]
    use pyo3::prelude::*;
    use serde::Serialize;

    use crate::{
        cache::buf::Buffer,
        types::variables::{Varbit, Varp, VarpOrVarbit},
    };

    /// Contains an array of possible ids this npc can morph into, controlled by either a varbit or varp.
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct NpcMorphTable {
        #[serde(flatten)]
        pub var: VarpOrVarbit,

        pub ids: Vec<Option<u32>>,
    }

    impl NpcMorphTable {
        /// Constructor for [`NpcMorphTable`]
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let varbit = Varbit::new(buffer.read_unsigned_short());
            let varp = Varp::new(buffer.read_unsigned_short());
            let var = VarpOrVarbit::new(varp, varbit);

            let count = buffer.read_unsigned_smart() as usize;
            let ids = iter::repeat_with(|| match buffer.read_unsigned_short() {
                u16::MAX => None,
                id => Some(id as u32),
            })
            .take(count + 1)
            .collect::<Vec<_>>();

            Self { var, ids }
        }
    }
    /// Like [`NpcMorphTable`], but with a default value.
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct ExtendedNpcMorphTable {
        pub var: VarpOrVarbit,

        pub ids: Vec<Option<u32>>,

        pub default_id: Option<u32>,
    }

    impl ExtendedNpcMorphTable {
        /// Constructor for [`ExtendedNpcMorphTable`]
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let varbit = Varbit::new(buffer.read_unsigned_short());
            let varp = Varp::new(buffer.read_unsigned_short());

            let var = VarpOrVarbit::new(varp, varbit);

            let default_id = buffer.read_smart32();

            let count = buffer.read_unsigned_smart() as usize;

            let ids = iter::repeat_with(|| match buffer.read_unsigned_short() {
                u16::MAX => None,
                id => Some(id as u32),
            })
            .take(count + 1)
            .collect::<Vec<_>>();

            Self { var, ids, default_id }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct NpcModels {
        pub models: Vec<Option<u32>>,
    }

    impl NpcModels {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_byte() as usize;

            let models = iter::repeat_with(|| buffer.read_smart32()).take(count).collect::<Vec<_>>();
            Self { models }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct ShadowIntensity {
        pub src_colour: i8,

        pub dst_colour: i8,
    }

    impl ShadowIntensity {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let src_colour = buffer.read_byte();
            let dst_colour = buffer.read_byte();
            Self { src_colour, dst_colour }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct Shadow {
        pub src_colour: u16,

        pub dst_colour: u16,
    }

    impl Shadow {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let src_colour = buffer.read_unsigned_short();
            let dst_colour = buffer.read_unsigned_short();
            Self { src_colour, dst_colour }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct HeadModels {
        pub headmodels: Vec<Option<u32>>,
    }

    impl HeadModels {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_unsigned_byte() as usize;
            let headmodels = iter::repeat_with(|| buffer.read_smart32()).take(count).collect::<Vec<_>>();
            Self { headmodels }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct ColourReplacements {
        pub colour_replacements: Vec<(u16, u16)>,
    }

    impl ColourReplacements {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_unsigned_byte() as usize;
            let colour_replacements = iter::repeat_with(|| (buffer.read_unsigned_short(), buffer.read_unsigned_short()))
                .take(count)
                .collect::<Vec<_>>();
            Self { colour_replacements }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct Textures {
        pub textures: BTreeMap<u16, u16>,
    }

    impl Textures {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_unsigned_byte() as usize;
            let textures = iter::repeat_with(|| (buffer.read_unsigned_short(), buffer.read_unsigned_short()))
                .take(count)
                .collect::<BTreeMap<_, _>>();
            Self { textures }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Debug, Serialize, Clone, Copy)]
    pub struct AmbientSounds {
        pub unknown_1: u16,

        pub unknown_2: u16,

        pub unknown_3: u16,

        pub unknown_4: u16,

        pub unknown_5: u8,
    }

    impl AmbientSounds {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let unknown_1 = buffer.read_unsigned_short();
            let unknown_2 = buffer.read_unsigned_short();
            let unknown_3 = buffer.read_unsigned_short();
            let unknown_4 = buffer.read_unsigned_short();
            let unknown_5 = buffer.read_unsigned_byte();

            Self {
                unknown_1,
                unknown_2,
                unknown_3,
                unknown_4,
                unknown_5,
            }
        }
    }
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Debug, Serialize, Clone)]
    pub struct Translations {
        pub translations: Vec<[u8; 4]>,
    }

    impl Translations {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_unsigned_byte() as usize;
            let translations = iter::repeat_with(|| {
                [
                    buffer.read_unsigned_byte(),
                    buffer.read_unsigned_byte(),
                    buffer.read_unsigned_byte(),
                    buffer.read_unsigned_byte(),
                ]
            })
            .take(count)
            .collect::<Vec<_>>();

            Self { translations }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Debug, Serialize, Clone)]
    pub struct RecolourPalette {
        pub recolour_palette: Vec<i8>,
    }

    impl RecolourPalette {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_unsigned_byte() as usize;

            let recolour_palette = iter::repeat_with(|| buffer.read_byte()).take(count).collect::<Vec<_>>();
            Self { recolour_palette }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Debug, Serialize, Clone, Copy)]
    pub struct OldCursors {
        pub op: u8,

        pub cursor: u16,
    }

    impl OldCursors {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let op = buffer.read_unsigned_byte();
            let cursor = buffer.read_unsigned_short();
            Self { op, cursor }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Debug, Serialize, Clone, Copy)]
    pub struct Unknown155 {
        pub unknown_1: i8,
        pub unknown_2: i8,
        pub unknown_3: i8,
        pub unknown_4: i8,
    }

    impl Unknown155 {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let unknown_1 = buffer.read_byte();
            let unknown_2 = buffer.read_byte();
            let unknown_3 = buffer.read_byte();
            let unknown_4 = buffer.read_byte();

            Self {
                unknown_1,
                unknown_2,
                unknown_3,
                unknown_4,
            }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Debug, Serialize, Clone, Copy)]
    pub struct Unknown179 {
        pub unknown_1: u16,
        pub unknown_2: u16,
        pub unknown_3: u16,
        pub unknown_4: u16,
        pub unknown_5: u16,
        pub unknown_6: u16,
    }

    impl Unknown179 {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let unknown_1 = buffer.read_unsigned_smart();
            let unknown_2 = buffer.read_unsigned_smart();
            let unknown_3 = buffer.read_unsigned_smart();
            let unknown_4 = buffer.read_unsigned_smart();
            let unknown_5 = buffer.read_unsigned_smart();
            let unknown_6 = buffer.read_unsigned_smart();

            Self {
                unknown_1,
                unknown_2,
                unknown_3,
                unknown_4,
                unknown_5,
                unknown_6,
            }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Debug, Serialize, Clone, Copy)]
    pub struct Unknown164 {
        pub unknown_1: u16,
        pub unknown_2: u16,
    }

    impl Unknown164 {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let unknown_1 = buffer.read_unsigned_short();
            let unknown_2 = buffer.read_unsigned_short();

            Self { unknown_1, unknown_2 }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Debug, Serialize, Clone)]
    pub struct Quests {
        pub quests: Vec<u16>,
    }

    impl Quests {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_unsigned_byte() as usize;
            let quests = iter::repeat_with(|| buffer.read_unsigned_short()).take(count).collect::<Vec<_>>();
            Self { quests }
        }
    }
}

use npc_config_fields::*;

/// Save the npc configs as `npc_configs.json`. Exposed as `--dump npc_configs`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output)?;
    let mut npc_configs = NpcConfig::dump_all(config)?.into_values().collect::<Vec<_>>();
    npc_configs.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(path!(config.output / "npc_configs.json"))?;
    let data = serde_json::to_string_pretty(&npc_configs).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_hans() -> CacheResult<()> {
        let config = crate::cli::Config::default();

        let npc_config = NpcConfig::dump_all(&config)?;
        let npc = npc_config.get(&0).unwrap();
        let name = npc.name.as_ref().unwrap();
        assert_eq!(name, "Hans", "{:?}", npc);
        Ok(())
    }
}
