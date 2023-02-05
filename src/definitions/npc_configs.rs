use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};

use bytes::{Buf, Bytes};
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rs3cache_backend::{buf::JString, error::Context};
use serde::Serialize;
#[cfg(any(feature = "rs3", feature = "osrs"))]
use {crate::definitions::indextype::IndexType, rs3cache_backend::index::CacheIndex};

#[cfg(feature = "osrs")]
use crate::definitions::indextype::ConfigType;
use crate::{
    cache::{buf::BufExtra, error::CacheResult},
    structures::paramtable::ParamTable,
};

/// Describes the properties of a given [`Npc`](crate::definitions::npcs::Npc).

#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", pyclass(frozen, get_all))]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct NpcConfig {
    /// Its id.
    pub id: u32,
    #[serde(flatten)]
    pub models: Option<NpcModels>,
    pub name: Option<JString<Bytes>>,
    pub size: Option<u8>,
    #[cfg(feature = "osrs")]
    pub standing_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub idle_90_left_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub idle_90_right_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub walking_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub rotate_180_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub rotate_90_right_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub rotate_90_left_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub run_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub run_90_right_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub run_90_left_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub run_180_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub crawl_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub crawl_90_right_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub crawl_90_left_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub crawl_180_animation: Option<u16>,
    #[cfg(feature = "osrs")]
    pub category: Option<u16>,

    pub actions: Option<[Option<JString<Bytes>>; 5]>,
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
    #[cfg(feature = "rs3")]
    pub head_icon_data: Option<Vec<(Option<u32>, Option<u32>)>>,
    #[cfg(feature = "osrs")]
    pub head_icon_data: Option<u16>,
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
    pub member_actions: Option<[Option<JString<Bytes>>; 5]>,
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
    #[cfg(feature = "rs3")]
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::NPC_CONFIG, config.input.clone())?.into_iter();

        let npc_configs = archives
            .map(Result::unwrap)
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

    #[cfg(feature = "osrs")]
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        Ok(CacheIndex::new(IndexType::CONFIG, config.input.clone())?
            .archive(ConfigType::NPC_CONFIG)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, Self::deserialize(file_id, file)))
            .collect())
    }

    #[cfg(feature = "legacy")]
    pub fn dump_all(_config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        todo!()
    }

    pub fn deserialize(id: u32, mut buffer: Bytes) -> Self {
        let mut npc = Self { id, ..Default::default() };

        loop {
            match buffer.get_u8() {
                0 => {
                    debug_assert_eq!(buffer.remaining(), 0, "The buffer was not fully read. {npc}");
                    break npc;
                }
                1 => npc.models = Some(NpcModels::deserialize(&mut buffer)),
                2 => npc.name = Some(buffer.get_string()),
                12 => npc.size = Some(buffer.get_u8()),
                #[cfg(feature = "osrs")]
                13 => npc.standing_animation = Some(buffer.get_u16()),
                #[cfg(feature = "osrs")]
                14 => npc.walking_animation = Some(buffer.get_u16()),
                #[cfg(feature = "osrs")]
                15 => npc.idle_90_left_animation = Some(buffer.get_u16()),
                #[cfg(feature = "osrs")]
                16 => npc.idle_90_right_animation = Some(buffer.get_u16()),
                #[cfg(feature = "osrs")]
                17 => {
                    npc.walking_animation = Some(buffer.get_u16());
                    npc.rotate_180_animation = Some(buffer.get_u16());
                    npc.rotate_90_right_animation = Some(buffer.get_u16());
                    npc.rotate_90_left_animation = Some(buffer.get_u16());
                }
                #[cfg(feature = "osrs")]
                18 => npc.category = Some(buffer.get_u16()),
                opcode @ 30..=34 => {
                    let actions = npc.actions.get_or_insert([None, None, None, None, None]);
                    actions[opcode as usize - 30] = Some(buffer.get_string());
                }
                40 => npc.colour_replacements = Some(ColourReplacements::deserialize(&mut buffer)),
                41 => npc.texture_replacements = Some(Textures::deserialize(&mut buffer)),
                42 => npc.recolour_palette = Some(RecolourPalette::deserialize(&mut buffer)),
                44 => npc.recolour_indices = Some(buffer.get_masked_index()),
                45 => npc.retexture_indices = Some(buffer.get_masked_index()),
                60 => npc.head_models = Some(HeadModels::deserialize(&mut buffer)),
                93 => npc.draw_map_dot = Some(false),
                95 => npc.combat = Some(buffer.get_u16()),
                97 => npc.scale_xz = Some(buffer.get_u16()),
                98 => npc.scale_y = Some(buffer.get_u16()),
                99 => npc.unknown_99 = Some(false),
                100 => npc.ambience = Some(buffer.get_i8()),
                101 => npc.ambience = Some(buffer.get_i8()),
                #[cfg(feature = "rs3")]
                102 => npc.head_icon_data = Some(buffer.get_masked_data()),
                #[cfg(feature = "osrs")]
                102 => npc.head_icon_data = Some(buffer.get_u16()),
                103 => npc.unknown_103 = Some(buffer.get_u16()),
                106 => npc.morphs_1 = Some(NpcMorphTable::deserialize(&mut buffer)),
                107 => npc.unknown_107 = Some(false),
                109 => npc.slow_walk = Some(false),
                111 => npc.animate_idle = Some(false),
                113 => npc.shadow = Some(Shadow::deserialize(&mut buffer)),
                #[cfg(feature = "rs3")]
                114 => npc.shadow_alpha_intensity = Some(ShadowIntensity::deserialize(&mut buffer)),
                #[cfg(feature = "osrs")]
                114 => npc.run_animation = Some(buffer.get_u16()),
                #[cfg(feature = "osrs")]
                115 => {
                    npc.run_animation = Some(buffer.get_u16());
                    npc.run_180_animation = Some(buffer.get_u16());
                    npc.run_90_left_animation = Some(buffer.get_u16());
                    npc.run_90_right_animation = Some(buffer.get_u16());
                }
                #[cfg(feature = "osrs")]
                116 => npc.crawl_animation = Some(buffer.get_u16()),
                #[cfg(feature = "osrs")]
                117 => {
                    npc.crawl_animation = Some(buffer.get_u16());
                    npc.crawl_180_animation = Some(buffer.get_u16());
                    npc.crawl_90_left_animation = Some(buffer.get_u16());
                    npc.crawl_90_right_animation = Some(buffer.get_u16());
                }
                118 => npc.morphs_2 = Some(ExtendedNpcMorphTable::deserialize(&mut buffer)),
                119 => npc.movement_capabilities = Some(buffer.get_i8()),
                121 => npc.translations = Some(Translations::deserialize(&mut buffer)),
                123 => npc.icon_height = Some(buffer.get_u16()),
                125 => npc.respawn_direction = Some(buffer.get_i8()),
                127 => npc.animation_group = Some(buffer.get_u16()),
                128 => npc.movement_type = Some(buffer.get_i8()),
                134 => npc.ambient_sound = Some(AmbientSounds::deserialize(&mut buffer)),
                135 => npc.old_cursor = Some(OldCursors::deserialize(&mut buffer)),
                136 => npc.old_cursor_2 = Some(OldCursors::deserialize(&mut buffer)),
                137 => npc.attack_cursor = Some(buffer.get_u16()),
                138 => npc.army_icon = Some(buffer.get_smart32().unwrap()),
                140 => npc.unknown_140 = Some(buffer.get_u8()),
                141 => npc.animate_idle = Some(true),
                142 => npc.mapfunction = Some(buffer.get_u16()),
                143 => npc.unknown_143 = Some(true),
                opcode @ 150..=154 => {
                    let actions = npc.member_actions.get_or_insert([None, None, None, None, None]);
                    actions[opcode as usize - 150] = Some(buffer.get_string());
                }
                155 => npc.unknown_155 = Some(Unknown155::deserialize(&mut buffer)),
                158 => npc.unknown_158 = Some(true),
                159 => npc.unknown_159 = Some(false),
                160 => npc.quests = Some(Quests::deserialize(&mut buffer)),
                162 => npc.unknown_162 = Some(true),
                163 => npc.unknown_163 = Some(buffer.get_u8()),
                164 => npc.unknown_164 = Some(Unknown164::deserialize(&mut buffer)),
                165 => npc.unknown_165 = Some(buffer.get_u8()),
                168 => npc.unknown_168 = Some(buffer.get_u8()),
                169 => npc.unknown_169 = Some(false),
                opcode @ 170..=175 => {
                    let actions = npc.action_cursors.get_or_insert([None, None, None, None, None, None]);
                    actions[opcode as usize - 170] = Some(buffer.get_u16());
                }
                178 => npc.unknown_178 = Some(true),
                179 => npc.unknown_179 = Some(Unknown179::deserialize(&mut buffer)),
                180 => unimplemented!(),
                181 => unimplemented!(),
                182 => npc.unknown_182 = Some(true),
                184 => npc.unknown_184 = Some(buffer.get_unsigned_smart()),
                249 => npc.params = Some(ParamTable::deserialize(&mut buffer)),
                missing => {
                    unimplemented!("NpcConfig::deserialize cannot deserialize opcode {} in npc: \n {}\n", missing, npc)
                }
            }
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl NpcConfig {
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

    use bytes::{Buf, Bytes};
    #[cfg(feature = "pyo3")]
    use pyo3::prelude::*;
    use serde::Serialize;

    use crate::{
        cache::buf::BufExtra,
        types::variables::{Varbit, Varp, VarpOrVarbit},
    };

    /// Contains an array of possible ids this npc can morph into, controlled by either a varbit or varp.
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct NpcMorphTable {
        #[serde(flatten)]
        pub var: VarpOrVarbit,

        pub ids: Vec<Option<u32>>,
    }

    impl NpcMorphTable {
        /// Constructor for [`NpcMorphTable`]
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let varbit = Varbit::new(buffer.get_u16());
            let varp = Varp::new(buffer.get_u16());
            let var = VarpOrVarbit::new(varp, varbit);

            let count = if cfg!(feature = "rs3") {
                buffer.get_unsigned_smart() as usize
            } else {
                buffer.get_u8() as usize
            };

            let ids = iter::repeat_with(|| match buffer.get_u16() {
                u16::MAX => None,
                id => Some(id as u32),
            })
            .take(count + 1)
            .collect::<Vec<_>>();

            Self { var, ids }
        }
    }
    /// Like [`NpcMorphTable`], but with a default value.
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct ExtendedNpcMorphTable {
        pub var: VarpOrVarbit,

        pub ids: Vec<Option<u32>>,

        pub default_id: Option<u32>,
    }

    impl ExtendedNpcMorphTable {
        /// Constructor for [`ExtendedNpcMorphTable`]
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let varbit = Varbit::new(buffer.get_u16());
            let varp = Varp::new(buffer.get_u16());

            let var = VarpOrVarbit::new(varp, varbit);

            let default_id = buffer.get_smart32();

            let count = if cfg!(feature = "rs3") {
                buffer.get_unsigned_smart() as usize
            } else {
                buffer.get_u8() as usize
            };

            let ids = iter::repeat_with(|| match buffer.get_u16() {
                u16::MAX => None,
                id => Some(id as u32),
            })
            .take(count + 1)
            .collect::<Vec<_>>();

            Self { var, ids, default_id }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct NpcModels {
        pub models: Vec<Option<u32>>,
    }

    impl NpcModels {
        #[cfg(feature = "rs3")]
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let count = buffer.get_i8() as usize;

            let models = iter::repeat_with(|| buffer.get_smart32()).take(count).collect::<Vec<_>>();
            Self { models }
        }

        #[cfg(any(feature = "osrs", feature = "legacy"))]
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let count = buffer.get_u8() as usize;

            let models = iter::repeat_with(|| match buffer.get_u16() {
                u16::MAX => None,
                other => Some(other as u32),
            })
            .take(count)
            .collect::<Vec<_>>();
            Self { models }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct ShadowIntensity {
        pub src_colour: i8,

        pub dst_colour: i8,
    }

    impl ShadowIntensity {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let src_colour = buffer.get_i8();
            let dst_colour = buffer.get_i8();
            Self { src_colour, dst_colour }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct Shadow {
        pub src_colour: u16,

        pub dst_colour: u16,
    }

    impl Shadow {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let src_colour = buffer.get_u16();
            let dst_colour = buffer.get_u16();
            Self { src_colour, dst_colour }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct HeadModels {
        #[cfg(feature = "rs3")]
        pub models: Vec<Option<u32>>,
        #[cfg(any(feature = "osrs", feature = "legacy"))]
        pub models: Vec<Option<u16>>,
    }

    impl HeadModels {
        #[cfg(feature = "rs3")]
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let count = buffer.get_i8() as usize;

            let models = iter::repeat_with(|| buffer.get_smart32()).take(count).collect::<Vec<_>>();
            Self { models }
        }

        #[cfg(any(feature = "osrs", feature = "legacy"))]
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let count = buffer.get_u8() as usize;

            let models = iter::repeat_with(|| match buffer.get_u16() {
                u16::MAX => None,
                other => Some(other),
            })
            .take(count)
            .collect::<Vec<_>>();
            Self { models }
        }
    }
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct ColourReplacements {
        pub colour_replacements: Vec<(u16, u16)>,
    }

    impl ColourReplacements {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let count = buffer.get_u8() as usize;
            let colour_replacements = iter::repeat_with(|| (buffer.get_u16(), buffer.get_u16())).take(count).collect::<Vec<_>>();
            Self { colour_replacements }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct Textures {
        pub textures: BTreeMap<u16, u16>,
    }

    impl Textures {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let count = buffer.get_u8() as usize;
            let textures = iter::repeat_with(|| (buffer.get_u16(), buffer.get_u16()))
                .take(count)
                .collect::<BTreeMap<_, _>>();
            Self { textures }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Debug, Serialize, Clone, Copy)]
    pub struct AmbientSounds {
        pub unknown_1: u16,

        pub unknown_2: u16,

        pub unknown_3: u16,

        pub unknown_4: u16,

        pub unknown_5: u8,
    }

    impl AmbientSounds {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let unknown_1 = buffer.get_u16();
            let unknown_2 = buffer.get_u16();
            let unknown_3 = buffer.get_u16();
            let unknown_4 = buffer.get_u16();
            let unknown_5 = buffer.get_u8();

            Self {
                unknown_1,
                unknown_2,
                unknown_3,
                unknown_4,
                unknown_5,
            }
        }
    }
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Debug, Serialize, Clone)]
    pub struct Translations {
        pub translations: Vec<[u8; 4]>,
    }

    impl Translations {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let count = buffer.get_u8() as usize;
            let translations = iter::repeat_with(|| [buffer.get_u8(), buffer.get_u8(), buffer.get_u8(), buffer.get_u8()])
                .take(count)
                .collect::<Vec<_>>();

            Self { translations }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Debug, Serialize, Clone)]
    pub struct RecolourPalette {
        pub recolour_palette: Vec<i8>,
    }

    impl RecolourPalette {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let count = buffer.get_u8() as usize;

            let recolour_palette = iter::repeat_with(|| buffer.get_i8()).take(count).collect::<Vec<_>>();
            Self { recolour_palette }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Debug, Serialize, Clone, Copy)]
    pub struct OldCursors {
        pub op: u8,

        pub cursor: u16,
    }

    impl OldCursors {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let op = buffer.get_u8();
            let cursor = buffer.get_u16();
            Self { op, cursor }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Debug, Serialize, Clone, Copy)]
    pub struct Unknown155 {
        pub unknown_1: i8,
        pub unknown_2: i8,
        pub unknown_3: i8,
        pub unknown_4: i8,
    }

    impl Unknown155 {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let unknown_1 = buffer.get_i8();
            let unknown_2 = buffer.get_i8();
            let unknown_3 = buffer.get_i8();
            let unknown_4 = buffer.get_i8();

            Self {
                unknown_1,
                unknown_2,
                unknown_3,
                unknown_4,
            }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
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
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let unknown_1 = buffer.get_unsigned_smart();
            let unknown_2 = buffer.get_unsigned_smart();
            let unknown_3 = buffer.get_unsigned_smart();
            let unknown_4 = buffer.get_unsigned_smart();
            let unknown_5 = buffer.get_unsigned_smart();
            let unknown_6 = buffer.get_unsigned_smart();

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

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Debug, Serialize, Clone, Copy)]
    pub struct Unknown164 {
        pub unknown_1: u16,
        pub unknown_2: u16,
    }

    impl Unknown164 {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let unknown_1 = buffer.get_u16();
            let unknown_2 = buffer.get_u16();

            Self { unknown_1, unknown_2 }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Debug, Serialize, Clone)]
    pub struct Quests {
        pub quests: Vec<u16>,
    }

    impl Quests {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let count = buffer.get_u8() as usize;
            let quests = iter::repeat_with(|| buffer.get_u16()).take(count).collect::<Vec<_>>();
            Self { quests }
        }
    }
}

use npc_config_fields::*;

/// Save the npc configs as `npc_configs.json`. Exposed as `--dump npc_configs`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output).context(&config.output)?;
    let mut npc_configs = NpcConfig::dump_all(config)?.into_values().collect::<Vec<_>>();
    npc_configs.sort_unstable_by_key(|loc| loc.id);
    let path = path!(config.output / "npc_configs.json");

    let mut file = File::create(&path).context(path.clone())?;

    let data = serde_json::to_string_pretty(&npc_configs).unwrap();
    file.write_all(data.as_bytes()).context(path)?;

    Ok(())
}

#[cfg(feature = "rs3")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_hans() -> CacheResult<()> {
        let config = crate::cli::Config::env();

        let npc_config = NpcConfig::dump_all(&config)?;
        let npc = npc_config.get(&0).unwrap();
        let name = npc.name.as_ref().unwrap();
        assert_eq!(name, "Hans", "{npc:?}");
        Ok(())
    }
}
