use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};

use fstrings::{f, format_args_f};
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, PyObjectProtocol};
use rayon::iter::{ParallelBridge, ParallelIterator};
#[cfg(feature = "osrs")]
use rs3cache_core::indextype::ConfigType;
use rs3cache_core::{buf::Buffer, error::CacheResult, index::CacheIndex, indextype::IndexType};
use serde::Serialize;

use crate::structures::paramtable::ParamTable;

/// Describes the properties of a given [`Location`](crate::definitions::locations::Location).
#[cfg_eval]
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct LocationConfig {
    /// Its id.
    pub id: u32,
    /// A mapping of possible types to models.
    #[serde(flatten)]
    pub models: Option<Models>,
    /// Its name, if present.
    pub name: Option<String>,
    #[cfg(feature = "osrs")]
    pub models_2: Option<Models2>,
    /// Its west-east dimension, defaulting to 1 if not present.
    ///
    /// Code using this value must account for the location's rotation.
    pub dim_x: Option<u8>,
    /// Its south-north dimension, defaulting to 1 if not present.
    ///
    /// Code using this value must account for the location's rotation.
    pub dim_y: Option<u8>,
    pub unknown_17: Option<bool>,
    pub is_transparent: Option<bool>,
    /// Flag for whether this object has a red rather than a white line on the map.
    pub unknown_19: Option<u8>,
    pub unknown_21: Option<bool>,
    pub unknown_22: Option<bool>,
    pub occludes_1: Option<bool>,
    pub unknown_24: Option<u32>,
    pub unknown_27: Option<bool>,
    pub unknown_28: Option<u8>,
    pub ambient: Option<i8>,
    /// What rightclick options this location has, if any.
    pub actions: Option<[Option<String>; 5]>,
    pub contrast: Option<i8>,
    #[serde(flatten)]
    pub colour_replacements: Option<ColourReplacements>,
    #[serde(flatten)]
    pub textures: Option<Textures>,
    pub recolour_palette: Option<Vec<(u16, u16)>>,
    pub unknown_44: Option<u16>,
    pub unknown_45: Option<u16>,
    #[cfg(feature = "osrs")]
    pub category: Option<u16>,
    pub mirror: Option<bool>,
    pub model: Option<bool>,
    pub scale_x: Option<u16>,
    pub scale_y: Option<u16>,
    pub scale_z: Option<u16>,
    pub unknown_69: Option<u8>,
    pub translate_x: Option<u16>,
    pub translate_y: Option<u16>,
    pub translate_z: Option<u16>,
    pub unknown_73: Option<bool>,
    /// Whether this location can be interacted through with e.g. ranged/magic combat, telegrab etc.
    pub blocks_ranged: Option<bool>,
    pub unknown_75: Option<u8>,
    /// This location can have different appearances depending on a player's varp/varbits.
    pub morphs_1: Option<LocationMorphTable>,
    pub unknown_78: Option<Unknown78>,
    pub unknown_79: Option<Unknown79>,
    pub unknown_81: Option<u8>,
    #[cfg(feature = "rs3")]
    pub unknown_82: Option<bool>,
    #[cfg(feature = "osrs")]
    pub maparea_id: Option<u16>,
    pub unknown_88: Option<bool>,
    pub unknown_89: Option<bool>,
    pub is_members: Option<bool>,
    /// This location can have different appearances depending on a players varbits,
    /// like the [morphs_1](LocationConfig::morphs_1) field, but with a default value.
    pub morphs_2: Option<ExtendedLocationMorphTable>,
    pub unknown_93: Option<u16>,
    pub unknown_94: Option<bool>,
    pub unknown_95: Option<u16>,
    pub unknown_96: Option<bool>,
    pub unknown_97: Option<bool>,
    pub unknown_98: Option<bool>,
    pub unknown_99: Option<()>,
    pub unknown_101: Option<u8>,
    /// Reference to a [`MapScene`](super::mapscenes::MapScene) that is drawn on the map.
    pub mapscene: Option<u16>,
    pub occludes_2: Option<bool>,
    pub unknown_104: Option<u8>,
    pub headmodels: Option<HeadModels>,
    pub mapfunction: Option<u16>,
    pub member_actions: Option<[Option<String>; 5]>,
    pub unknown_160: Option<Unknown160>,
    pub unknown_162: Option<i32>,
    pub unknown_163: Option<Unknown163>,
    pub unknown_164: Option<u16>,
    pub unknown_165: Option<u16>,
    pub unknown_166: Option<u16>,
    pub unknown_167: Option<u16>,
    pub unknown_168: Option<bool>,
    pub unknown_169: Option<bool>,
    pub unknown_170: Option<u16>,
    pub unknown_171: Option<u16>,
    #[serde(flatten)]
    pub unknown_173: Option<Unknown173>,
    pub unknown_177: Option<bool>,
    pub unknown_178: Option<u8>,
    pub unknown_186: Option<u8>,
    pub unknown_188: Option<bool>,
    pub unknown_189: Option<bool>,
    pub cursors: Option<[Option<u16>; 6]>,
    pub unknown_196: Option<u8>,
    pub unknown_197: Option<u8>,
    pub unknown_198: Option<bool>,
    pub unknown_199: Option<bool>,
    pub unknown_200: Option<bool>,
    #[serde(flatten)]
    pub unknown_201: Option<Unknown201>,
    pub unknown_202: Option<u16>,
    #[serde(flatten)]
    pub params: Option<ParamTable>,
}

impl LocationConfig {
    /// Returns a mapping of all [location configurations](LocationConfig)
    #[cfg(feature = "rs3")]
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::LOC_CONFIG, &config.input)?.into_iter();
        let locations = archives
            .map(Result::unwrap)
            .flat_map(|archive| {
                let archive_id = archive.archive_id();
                archive
                    .take_files()
                    .into_iter()
                    .map(move |(file_id, file)| (archive_id << 8 | file_id, file))
            })
            .map(|(id, file)| (id, Self::deserialize(id, file)))
            .collect::<BTreeMap<u32, Self>>();
        Ok(locations)
    }

    #[cfg(feature = "osrs")]
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        Ok(CacheIndex::new(IndexType::CONFIG, &config.input)?
            .archive(ConfigType::LOC_CONFIG)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, Self::deserialize(file_id, file)))
            .collect())
    }

    fn deserialize(id: u32, file: Vec<u8>) -> Self {
        let mut buffer = Buffer::new(file);
        let mut loc = Self { id, ..Default::default() };

        loop {
            match buffer.read_unsigned_byte() {
                0 => {
                    debug_assert_eq!(buffer.remaining(), 0);
                    break loc;
                }
                1 => loc.models = Some(Models::deserialize(&mut buffer)),
                2 => loc.name = Some(buffer.read_string()),
                #[cfg(feature = "osrs")]
                5 => loc.models_2 = Some(Models2::deserialize(&mut buffer)),
                14 => loc.dim_x = Some(buffer.read_unsigned_byte()),
                15 => loc.dim_y = Some(buffer.read_unsigned_byte()),
                17 => loc.unknown_17 = Some(false),
                18 => loc.is_transparent = Some(true),
                19 => loc.unknown_19 = Some(buffer.read_unsigned_byte()),
                21 => loc.unknown_21 = Some(true),
                22 => loc.unknown_22 = Some(true),
                23 => loc.occludes_1 = Some(false),
                24 => loc.unknown_24 = buffer.read_smart32(),
                27 => loc.unknown_27 = Some(false),
                28 => loc.unknown_28 = Some(buffer.read_unsigned_byte()),
                29 => loc.ambient = Some(buffer.read_byte()),
                opcode @ 30..=34 => {
                    let actions = loc.actions.get_or_insert([None, None, None, None, None]);
                    actions[opcode as usize - 30] = Some(buffer.read_string());
                }
                39 => loc.contrast = Some(buffer.read_byte()),
                40 => loc.colour_replacements = Some(ColourReplacements::deserialize(&mut buffer)),
                41 => loc.textures = Some(Textures::deserialize(&mut buffer)),
                44 => loc.unknown_44 = Some(buffer.read_masked_index()),
                45 => loc.unknown_45 = Some(buffer.read_masked_index()),
                // changed at some point after 2015
                // used to be mapscenes
                // see https://discordapp.com/channels/177206626514632704/269673599554551808/872603876384178206
                #[cfg(feature = "osrs")]
                60 => loc.mapscene = Some(buffer.read_unsigned_short()),

                #[cfg(feature = "osrs")]
                61 => loc.category = Some(buffer.read_unsigned_short()),
                62 => loc.mirror = Some(true),
                64 => loc.model = Some(false),
                65 => loc.scale_x = Some(buffer.read_unsigned_short()),
                66 => loc.scale_y = Some(buffer.read_unsigned_short()),
                67 => loc.scale_z = Some(buffer.read_unsigned_short()),
                #[cfg(feature = "osrs")]
                68 => loc.mapscene = Some(buffer.read_unsigned_short()),
                69 => loc.unknown_69 = Some(buffer.read_unsigned_byte()),
                70 => loc.translate_x = Some(buffer.read_unsigned_short()),
                71 => loc.translate_y = Some(buffer.read_unsigned_short()),
                72 => loc.translate_z = Some(buffer.read_unsigned_short()),
                73 => loc.unknown_73 = Some(true),
                74 => loc.blocks_ranged = Some(true),
                75 => loc.unknown_75 = Some(buffer.read_unsigned_byte()),
                77 => loc.morphs_1 = Some(LocationMorphTable::deserialize(&mut buffer)),
                78 => loc.unknown_78 = Some(Unknown78::deserialize(&mut buffer)),
                79 => loc.unknown_79 = Some(Unknown79::deserialize(&mut buffer)),
                81 => loc.unknown_81 = Some(buffer.read_unsigned_byte()),
                #[cfg(feature = "rs3")]
                82 => loc.unknown_82 = Some(true),
                #[cfg(feature = "osrs")]
                82 => loc.maparea_id = Some(buffer.read_unsigned_short()),
                88 => loc.unknown_88 = Some(false),
                89 => loc.unknown_89 = Some(false),
                91 => loc.is_members = Some(true),
                92 => loc.morphs_2 = Some(ExtendedLocationMorphTable::deserialize(&mut buffer)),
                93 => loc.unknown_93 = Some(buffer.read_unsigned_short()),
                94 => loc.unknown_94 = Some(true),
                95 => loc.unknown_95 = Some(buffer.read_unsigned_short()),
                97 => loc.unknown_97 = Some(true),
                98 => loc.unknown_98 = Some(true),
                #[cfg(feature = "rs3")]
                102 => loc.mapscene = Some(buffer.read_unsigned_short()),
                104 => loc.unknown_104 = Some(buffer.read_unsigned_byte()),
                106 => loc.headmodels = Some(HeadModels::deserialize(&mut buffer)),
                107 => loc.mapfunction = Some(buffer.read_unsigned_short()),
                103 => loc.occludes_2 = Some(false),
                opcode @ 150..=154 => {
                    let actions = loc.member_actions.get_or_insert([None, None, None, None, None]);
                    actions[opcode as usize - 150] = Some(buffer.read_string());
                }
                160 => loc.unknown_160 = Some(Unknown160::deserialize(&mut buffer)),
                162 => loc.unknown_162 = Some(buffer.read_int()),
                163 => loc.unknown_163 = Some(Unknown163::deserialize(&mut buffer)),
                164 => loc.unknown_164 = Some(buffer.read_unsigned_short()),
                165 => loc.unknown_166 = Some(buffer.read_unsigned_short()),
                167 => loc.unknown_167 = Some(buffer.read_unsigned_short()),
                170 => loc.unknown_170 = Some(buffer.read_unsigned_smart()),
                171 => loc.unknown_171 = Some(buffer.read_unsigned_smart()),
                173 => loc.unknown_173 = Some(Unknown173::deserialize(&mut buffer)),
                177 => loc.unknown_177 = Some(true),
                178 => loc.unknown_178 = Some(buffer.read_unsigned_byte()),
                186 => loc.unknown_186 = Some(buffer.read_unsigned_byte()),
                188 => loc.unknown_188 = Some(true),
                189 => loc.unknown_189 = Some(true),
                opcode @ 190..=195 => {
                    let actions = loc.cursors.get_or_insert([None, None, None, None, None, None]);
                    actions[opcode as usize - 190] = Some(buffer.read_unsigned_short());
                }
                196 => loc.unknown_196 = Some(buffer.read_unsigned_byte()),
                197 => loc.unknown_196 = Some(buffer.read_unsigned_byte()),
                198 => loc.unknown_198 = Some(true),
                199 => loc.unknown_199 = Some(true),
                201 => loc.unknown_201 = Some(Unknown201::deserialize(&mut buffer)),
                202 => loc.unknown_202 = Some(buffer.read_unsigned_smart()),
                249 => loc.params = Some(ParamTable::deserialize(&mut buffer)),
                missing => unimplemented!("LocationConfig::deserialize cannot deserialize opcode {} in id {}", missing, id),
            }
        }
    }
}

/// Defines the structs used as fields of [`LocationConfig`],
pub mod location_config_fields {
    #![allow(missing_docs)]
    use std::{collections::BTreeMap, iter};

    #[cfg(feature = "pyo3")]
    use pyo3::prelude::*;
    use serde::Serialize;

    use crate::{
        cache::buf::Buffer,
        types::variables::{Varbit, Varp, VarpOrVarbit},
    };
    /// Contains an array of possible ids this location can morph into, controlled by either a varbit or varp.
    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct LocationMorphTable {
        #[serde(flatten)]
        pub var: VarpOrVarbit,

        /// The possible ids this [`LocationConfig`](super::LocationConfig) can be.
        #[cfg(feature = "rs3")]
        pub ids: Vec<Option<u32>>,

        /// The possible ids this [`LocationConfig`](super::LocationConfig) can be.
        #[cfg(feature = "osrs")]
        pub ids: Vec<Option<u16>>,
    }

    impl LocationMorphTable {
        /// Constructor for [`LocationMorphTable`]
        #[cfg(feature = "rs3")]
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let varbit = Varbit::new(buffer.read_unsigned_short());
            let varp = Varp::new(buffer.read_unsigned_short());
            let var = VarpOrVarbit::new(varp, varbit);

            let count = buffer.read_unsigned_smart() as usize;

            let ids = iter::repeat_with(|| buffer.read_smart32()).take(count + 1).collect::<Vec<_>>();

            Self { var, ids }
        }

        #[cfg(feature = "osrs")]
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let varbit = Varbit::new(buffer.read_unsigned_short());
            let varp = Varp::new(buffer.read_unsigned_short());
            let var = VarpOrVarbit::new(varp, varbit);

            let count = buffer.read_unsigned_byte() as usize;

            let ids = iter::repeat_with(|| match buffer.read_unsigned_short() {
                0xFFFF => None,
                id => Some(id),
            })
            .take(count + 1)
            .collect::<Vec<_>>();

            Self { var, ids }
        }
    }

    /// Like [`LocationMorphTable`], but with a default value.
    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[allow(missing_docs)]
    #[derive(Serialize, Debug, Clone)]
    pub struct ExtendedLocationMorphTable {
        #[serde(flatten)]
        pub var: VarpOrVarbit,

        /// The possible ids this [`LocationConfig`](super::LocationConfig) can be.
        #[cfg(feature = "rs3")]
        pub ids: Vec<Option<u32>>,

        /// This [`LocationConfig`](super::LocationConfig)'s default id.
        #[cfg(feature = "rs3")]
        pub default: Option<u32>,

        /// The possible ids this [`LocationConfig`](super::LocationConfig) can be.
        #[cfg(feature = "osrs")]
        pub ids: Vec<Option<u16>>,

        /// This [`LocationConfig`](super::LocationConfig)'s default id.
        #[cfg(feature = "osrs")]
        pub default: Option<u16>,
    }

    impl ExtendedLocationMorphTable {
        /// Constructor for [`ExtendedLocationMorphTable`]
        #[cfg(feature = "rs3")]
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let varbit = Varbit::new(buffer.read_unsigned_short());
            let varp = Varp::new(buffer.read_unsigned_short());

            let var = VarpOrVarbit::new(varp, varbit);

            let default = buffer.read_smart32();

            let count = buffer.read_unsigned_smart() as usize;

            let ids = iter::repeat_with(|| buffer.read_smart32()).take(count + 1).collect::<Vec<_>>();
            Self { var, ids, default }
        }

        #[cfg(feature = "osrs")]
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let varbit = Varbit::new(buffer.read_unsigned_short());
            let varp = Varp::new(buffer.read_unsigned_short());

            let var = VarpOrVarbit::new(varp, varbit);

            let default = match buffer.read_unsigned_short() {
                0xFFFF => None,
                id => Some(id),
            };

            let count = buffer.read_unsigned_byte() as usize;

            let ids = iter::repeat_with(|| match buffer.read_unsigned_short() {
                0xFFFF => None,
                id => Some(id),
            })
            .take(count + 1)
            .collect::<Vec<_>>();
            Self { var, ids, default }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct ColourReplacements {
        pub colours: Vec<(u16, u16)>,
    }

    impl ColourReplacements {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_unsigned_byte() as usize;
            let colours = iter::repeat_with(|| (buffer.read_unsigned_short(), buffer.read_unsigned_short()))
                .take(count)
                .collect::<Vec<_>>();
            Self { colours }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct Models {
        #[cfg(feature = "rs3")]
        pub models: BTreeMap<i8, Vec<Option<u32>>>,
        #[cfg(feature = "osrs")]
        pub models: Vec<(u8, u16)>,
    }

    impl Models {
        #[cfg(feature = "rs3")]
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Models {
            let count = buffer.read_unsigned_byte() as usize;

            let models = iter::repeat_with(|| Models::sub_deserialize(buffer))
                .take(count)
                .collect::<BTreeMap<_, _>>();
            Models { models }
        }

        #[cfg(feature = "osrs")]
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Models {
            let count = buffer.read_unsigned_byte() as usize;

            let models = iter::repeat_with(|| {
                let model = buffer.read_unsigned_short();
                let r#type = buffer.read_unsigned_byte();
                (r#type, model)
            })
            .take(count)
            .collect::<Vec<_>>();
            Models { models }
        }

        #[cfg(feature = "rs3")]
        fn sub_deserialize(buffer: &mut Buffer<Vec<u8>>) -> (i8, Vec<Option<u32>>) {
            let ty = buffer.read_byte();
            let count = buffer.read_unsigned_byte() as usize;
            let values = iter::repeat_with(|| buffer.read_smart32()).take(count).collect::<Vec<_>>();
            (ty, values)
        }
    }

    #[cfg_eval]
    #[cfg(feature = "osrs")]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct Models2 {
        pub models: Vec<u16>,
    }

    #[cfg(feature = "osrs")]
    impl Models2 {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_unsigned_byte() as usize;

            let models = iter::repeat_with(|| buffer.read_unsigned_short()).take(count).collect();
            Self { models }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct Textures {
        pub textures: BTreeMap<u16, u16>,
    }

    impl Textures {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Textures {
            let count = buffer.read_unsigned_byte() as usize;
            let textures = iter::repeat_with(|| (buffer.read_unsigned_short(), buffer.read_unsigned_short()))
                .take(count)
                .collect::<BTreeMap<_, _>>();
            Textures { textures }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown79 {
        pub unknown_1: u16,

        pub unknown_2: u16,

        pub unknown_3: u8,

        pub values: Vec<u16>,
    }

    impl Unknown79 {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Unknown79 {
            let unknown_1 = buffer.read_unsigned_short();
            let unknown_2 = buffer.read_unsigned_short();
            let unknown_3 = buffer.read_unsigned_byte();

            let count = buffer.read_unsigned_byte() as usize;

            let values = iter::repeat_with(|| buffer.read_unsigned_short()).take(count).collect::<Vec<_>>();

            Unknown79 {
                unknown_1,
                unknown_2,
                unknown_3,
                values,
            }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct Unknown173 {
        pub unknown_1: u16,

        pub unknown_2: u16,
    }

    impl Unknown173 {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Unknown173 {
            let unknown_1 = buffer.read_unsigned_short();
            let unknown_2 = buffer.read_unsigned_short();

            Unknown173 { unknown_1, unknown_2 }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct Unknown163 {
        pub unknown_1: i8,

        pub unknown_2: i8,

        pub unknown_3: i8,

        pub unknown_4: i8,
    }

    impl Unknown163 {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Unknown163 {
            let unknown_1 = buffer.read_byte();
            let unknown_2 = buffer.read_byte();
            let unknown_3 = buffer.read_byte();
            let unknown_4 = buffer.read_byte();

            Unknown163 {
                unknown_1,
                unknown_2,
                unknown_3,
                unknown_4,
            }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct Unknown78 {
        pub unknown_1: u16,

        pub unknown_2: u8,
    }

    impl Unknown78 {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let unknown_1 = buffer.read_unsigned_short();
            let unknown_2 = buffer.read_unsigned_byte();

            Self { unknown_1, unknown_2 }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown160 {
        pub values: Vec<u16>,
    }

    impl Unknown160 {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Self {
            let count = buffer.read_unsigned_byte() as usize;
            let values = iter::repeat_with(|| buffer.read_unsigned_short()).take(count).collect::<Vec<_>>();
            Self { values }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct Unknown201 {
        pub unknown_1: u16,

        pub unknown_2: u16,

        pub unknown_3: u16,

        pub unknown_4: u16,

        pub unknown_5: u16,

        pub unknown_6: u16,
    }

    impl Unknown201 {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> Unknown201 {
            let unknown_1 = buffer.read_unsigned_smart();
            let unknown_2 = buffer.read_unsigned_smart();
            let unknown_3 = buffer.read_unsigned_smart();
            let unknown_4 = buffer.read_unsigned_smart();
            let unknown_5 = buffer.read_unsigned_smart();
            let unknown_6 = buffer.read_unsigned_smart();

            Unknown201 {
                unknown_1,
                unknown_2,
                unknown_3,
                unknown_4,
                unknown_5,
                unknown_6,
            }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct HeadModels {
        pub headmodels: Vec<(Option<u32>, u8)>,
    }

    impl HeadModels {
        pub fn deserialize(buffer: &mut Buffer<Vec<u8>>) -> HeadModels {
            let count = buffer.read_unsigned_byte() as usize;
            let headmodels = iter::repeat_with(|| (buffer.read_smart32(), buffer.read_unsigned_byte()))
                .take(count)
                .collect::<Vec<_>>();
            HeadModels { headmodels }
        }
    }
}

use location_config_fields::*;

/// Save the location configs as `location_configs.json`. Exposed as `--dump location_configs`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output)?;
    let mut loc_configs = LocationConfig::dump_all(config)?.into_values().collect::<Vec<_>>();
    loc_configs.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(path!(config.output / "location_configs.json"))?;
    let data = serde_json::to_string_pretty(&loc_configs).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}

///Save the location configs as individual `json` files.
pub fn export_each(config: &crate::cli::Config) -> CacheResult<()> {
    let folder = path!(&config.output / "location_configs");
    fs::create_dir_all(&folder)?;

    let configs = LocationConfig::dump_all(config)?;
    configs.into_iter().par_bridge().for_each(|(id, location_config)| {
        let mut file = File::create(path!(&folder / f!("{id}.json"))).unwrap();

        let data = serde_json::to_string_pretty(&location_config).unwrap();
        file.write_all(data.as_bytes()).unwrap();
    });

    Ok(())
}

#[cfg(feature = "pyo3")]
#[pyproto]
impl PyObjectProtocol for LocationConfig {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("LocationConfig({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("LocationConfig({})", serde_json::to_string(self).unwrap()))
    }
}

#[cfg(test)]
mod map_tests {
    use super::*;
    use crate::cli::Config;

    #[test]
    fn id_36687() -> CacheResult<()> {
        let config = Config::env();

        let loc_config = LocationConfig::dump_all(&config)?;
        let loc = loc_config.get(&36687).unwrap();
        let name = loc.name.as_ref().unwrap();

        #[cfg(feature = "rs3")]
        assert_eq!(name, "Trapdoor", "{:?}", loc);

        #[cfg(feature = "osrs")]
        assert_eq!(name, "Tree stump", "{:?}", loc);
        Ok(())
    }

    #[test]
    #[cfg(feature = "rs3")]
    fn check_paramtable() -> CacheResult<()> {
        use crate::structures::paramtable::Param;

        let config = Config::env();

        let loc_config = LocationConfig::dump_all(&config)?;
        let bookcase = loc_config.get(&118445).unwrap();
        let paramtable = bookcase.params.as_ref().unwrap();
        let value = &paramtable.params[&8178];
        assert_eq!(*value, Param::Integer(50923), "{:?}", paramtable);
        Ok(())
    }
}
