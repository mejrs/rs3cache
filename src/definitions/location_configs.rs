use crate::{
    cache::{buf::Buffer, index::CacheIndex, indextype::IndexType},
    structures::paramtable::ParamTable,
    utils::{error::CacheResult, par::ParApply},
};
use pyo3::{prelude::*, PyObjectProtocol};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

/// Describes the properties of a given [`Location`](crate::definitions::locations::Location).
#[allow(missing_docs)]
#[pyclass]
#[derive(Serialize, Debug, Default)]
pub struct LocationConfig {
    /// Its id.
    #[pyo3(get)]
    pub id: u32,

    /// A mapping of possible types to models.
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub models: Option<Models>,

    /// Its name, if present.
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Its west-east dimension, defaulting to 1 if not present.
    ///
    /// Code using this value must account for the location's rotation.
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim_x: Option<u8>,

    /// Its south-north dimension, defaulting to 1 if not present.
    ///
    /// Code using this value must account for the location's rotation.
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim_y: Option<u8>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_17: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_transparent: Option<bool>,

    /// Flag for whether this object has a red rather than a white line on the map.
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_19: Option<u8>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_21: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_22: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occludes_1: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_24: Option<u32>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_27: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_28: Option<u8>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambient: Option<i8>,

    /// What rightclick options this location has, if any.
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<[Option<String>; 5]>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contrast: Option<i8>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub colour_replacements: Option<ColourReplacements>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub textures: Option<Textures>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recolour_palette: Option<Vec<(u16, u16)>>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_44: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_45: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_x: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_y: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_z: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_69: Option<u8>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate_x: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate_y: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate_z: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_73: Option<bool>,

    /// Whether this location can be interacted through with e.g. ranged/magic combat, telegrab etc.
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks_ranged: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_75: Option<u8>,

    /// This location can have different appearances depending on a player's varp/varbits.
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub morphs_1: Option<LocationMorphTable>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_78: Option<Unknown78>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_79: Option<Unknown79>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_81: Option<u8>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_82: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_88: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_89: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_members: Option<bool>,

    /// This location can have different appearances depending on a players varbits,
    /// like the [morphs_1](LocationConfig::morphs_1) field, but with a default value.
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub morphs_2: Option<ExtendedLocationMorphTable>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_93: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_94: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_95: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_96: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_97: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_98: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_99: Option<()>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_101: Option<u8>,

    /// Reference to a [`MapScene`](super::mapscenes::MapScene) that is drawn on the map.
    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapscene: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occludes_2: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_104: Option<u8>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headmodels: Option<HeadModels>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapfunction: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_actions: Option<[Option<String>; 5]>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_160: Option<Unknown160>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_162: Option<i32>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_163: Option<Unknown163>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_164: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_165: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_166: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_167: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_168: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_169: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_170: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_171: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub unknown_173: Option<Unknown173>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_177: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_178: Option<u8>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_186: Option<u8>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_188: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_189: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursors: Option<[Option<u16>; 6]>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_196: Option<u8>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_197: Option<u8>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_198: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_199: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_200: Option<bool>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub unknown_201: Option<Unknown201>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_202: Option<u16>,

    #[pyo3(get)]
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub params: Option<ParamTable>,
}

impl LocationConfig {
    /// Returns a mapping of all [location configurations](LocationConfig)
    pub fn dump_all() -> CacheResult<HashMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::LOC_CONFIG)?.into_iter();

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
        let mut loc = Self { id, ..Default::default() };

        loop {
            match buffer.read_unsigned_byte() {
                0 => {
                    debug_assert_eq!(buffer.remaining(), 0);
                    break loc;
                }
                1 => loc.models = Some(Models::deserialize(&mut buffer)),
                2 => loc.name = Some(buffer.read_string()),
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
                62 => loc.mirror = Some(true),
                64 => loc.model = Some(false),
                65 => loc.scale_x = Some(buffer.read_unsigned_short()),
                66 => loc.scale_y = Some(buffer.read_unsigned_short()),
                67 => loc.scale_z = Some(buffer.read_unsigned_short()),
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
                82 => loc.unknown_82 = Some(true),
                88 => loc.unknown_88 = Some(false),
                89 => loc.unknown_89 = Some(false),
                91 => loc.is_members = Some(true),
                92 => loc.morphs_2 = Some(ExtendedLocationMorphTable::deserialize(&mut buffer)),
                93 => loc.unknown_93 = Some(buffer.read_unsigned_short()),
                94 => loc.unknown_94 = Some(true),
                95 => loc.unknown_95 = Some(buffer.read_unsigned_short()),
                97 => loc.unknown_97 = Some(true),
                98 => loc.unknown_98 = Some(true),
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

#[pyproto]
impl PyObjectProtocol for LocationConfig {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("LocationConfig({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("LocationConfig({})", serde_json::to_string(self).unwrap()))
    }
}

///Save the location configs as individual `json` files.
pub fn export_each() -> CacheResult<()> {
    fs::create_dir_all("out/data/rs3/location_configs")?;
    let configs = LocationConfig::dump_all()?;
    configs.into_iter().par_apply(|(id, config)| {
        let mut file = File::create(format!("out/data/rs3/location_configs/{}.json", id)).unwrap();
        let data = serde_json::to_string_pretty(&config).unwrap();
        file.write_all(data.as_bytes()).unwrap();
    });

    Ok(())
}

/// Defines the structs used as fields of [`LocationConfig`],
pub mod location_config_fields {
    #![allow(missing_docs)]
    use crate::{
        cache::buf::Buffer,
        types::variables::{Varbit, Varp, VarpOrVarbit},
    };
    use pyo3::prelude::*;
    use serde::Serialize;
    use std::{collections::HashMap, iter};
    /// Contains an array of possible ids this location can morph into, controlled by either a varbit or varp.
    #[pyclass]
    #[derive(Serialize, Debug, Clone)]
    pub struct LocationMorphTable {
        #[pyo3(get)]
        #[serde(flatten)]
        pub var: VarpOrVarbit,

        #[pyo3(get)]
        /// The possible ids this [`LocationConfig`](super::LocationConfig) can be.
        pub ids: Vec<Option<u32>>,
    }

    impl LocationMorphTable {
        /// Constructor for [`LocationMorphTable`]
        pub fn deserialize(buffer: &mut Buffer) -> Self {
            let varbit = Varbit::new(buffer.read_unsigned_short());
            let varp = Varp::new(buffer.read_unsigned_short());
            let var = VarpOrVarbit::new(varp, varbit);

            let count = buffer.read_unsigned_smart() as usize;
            let ids = iter::repeat_with(|| buffer.read_smart32()).take(count + 1).collect::<Vec<_>>();

            Self { var, ids }
        }
    }

    /// Like [`LocationMorphTable`], but with a default value.
    #[pyclass]
    #[allow(missing_docs)]
    #[derive(Serialize, Debug, Clone)]
    pub struct ExtendedLocationMorphTable {
        #[pyo3(get)]
        #[serde(flatten)]
        pub var: VarpOrVarbit,

        /// The possible ids this [`LocationConfig`](super::LocationConfig) can be.
        #[pyo3(get)]
        pub ids: Vec<Option<u32>>,

        /// This [`LocationConfig`](super::LocationConfig)'s default id.
        #[pyo3(get)]
        pub default: Option<u32>,
    }

    impl ExtendedLocationMorphTable {
        /// Constructor for [`ExtendedLocationMorphTable`]
        pub fn deserialize(buffer: &mut Buffer) -> Self {
            let varbit = Varbit::new(buffer.read_unsigned_short());
            let varp = Varp::new(buffer.read_unsigned_short());

            let var = VarpOrVarbit::new(varp, varbit);

            let default = buffer.read_smart32();

            let count = buffer.read_unsigned_smart() as usize;

            let ids = iter::repeat_with(|| buffer.read_smart32()).take(count + 1).collect::<Vec<_>>();
            Self { var, ids, default }
        }
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
    pub struct Models {
        #[pyo3(get)]
        pub models: HashMap<i8, Vec<Option<u32>>>,
    }

    impl Models {
        pub fn deserialize(buffer: &mut Buffer) -> Models {
            let count = buffer.read_unsigned_byte() as usize;

            let models = iter::repeat_with(|| Models::sub_deserialize(buffer))
                .take(count)
                .collect::<HashMap<_, _>>();
            Models { models }
        }

        fn sub_deserialize(buffer: &mut Buffer) -> (i8, Vec<Option<u32>>) {
            let ty = buffer.read_byte();
            let count = buffer.read_unsigned_byte() as usize;
            let values = iter::repeat_with(|| buffer.read_smart32()).take(count).collect::<Vec<_>>();
            (ty, values)
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
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown79 {
        #[pyo3(get)]
        pub unknown_1: u16,

        #[pyo3(get)]
        pub unknown_2: u16,

        #[pyo3(get)]
        pub unknown_3: u8,

        #[pyo3(get)]
        pub values: Vec<u16>,
    }

    impl Unknown79 {
        pub fn deserialize(buffer: &mut Buffer) -> Unknown79 {
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

    #[pyclass]
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown173 {
        #[pyo3(get)]
        pub unknown_1: u16,

        #[pyo3(get)]
        pub unknown_2: u16,
    }

    impl Unknown173 {
        pub fn deserialize(buffer: &mut Buffer) -> Unknown173 {
            let unknown_1 = buffer.read_unsigned_short();
            let unknown_2 = buffer.read_unsigned_short();

            Unknown173 { unknown_1, unknown_2 }
        }
    }

    #[pyclass]
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown163 {
        #[pyo3(get)]
        pub unknown_1: i8,

        #[pyo3(get)]
        pub unknown_2: i8,

        #[pyo3(get)]
        pub unknown_3: i8,

        #[pyo3(get)]
        pub unknown_4: i8,
    }

    impl Unknown163 {
        pub fn deserialize(buffer: &mut Buffer) -> Unknown163 {
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

    #[pyclass]
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown78 {
        #[pyo3(get)]
        pub unknown_1: u16,

        #[pyo3(get)]
        pub unknown_2: u8,
    }

    impl Unknown78 {
        pub fn deserialize(buffer: &mut Buffer) -> Unknown78 {
            let unknown_1 = buffer.read_unsigned_short();
            let unknown_2 = buffer.read_unsigned_byte();

            Unknown78 { unknown_1, unknown_2 }
        }
    }

    #[pyclass]
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown160 {
        #[pyo3(get)]
        pub values: Vec<u16>,
    }

    impl Unknown160 {
        pub fn deserialize(buffer: &mut Buffer) -> Unknown160 {
            let count = buffer.read_unsigned_byte() as usize;
            let values = iter::repeat_with(|| buffer.read_unsigned_short()).take(count).collect::<Vec<_>>();
            Unknown160 { values }
        }
    }

    #[pyclass]
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown201 {
        #[pyo3(get)]
        pub unknown_1: u16,

        #[pyo3(get)]
        pub unknown_2: u16,

        #[pyo3(get)]
        pub unknown_3: u16,

        #[pyo3(get)]
        pub unknown_4: u16,

        #[pyo3(get)]
        pub unknown_5: u16,

        #[pyo3(get)]
        pub unknown_6: u16,
    }

    impl Unknown201 {
        pub fn deserialize(buffer: &mut Buffer) -> Unknown201 {
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

    #[pyclass]
    #[derive(Serialize, Debug, Clone)]
    pub struct HeadModels {
        #[pyo3(get)]
        pub headmodels: Vec<(Option<u32>, u8)>,
    }

    impl HeadModels {
        pub fn deserialize(buffer: &mut Buffer) -> HeadModels {
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
pub fn export() -> CacheResult<()> {
    fs::create_dir_all("out")?;
    let mut loc_configs = LocationConfig::dump_all()?.into_values().collect::<Vec<_>>();
    loc_configs.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create("out/location_configs.json")?;
    let data = serde_json::to_string_pretty(&loc_configs).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod map_tests {
    use super::*;

    #[test]
    fn id_36687_is_trapdoor() -> CacheResult<()> {
        let loc_config = LocationConfig::dump_all()?;
        let trapdoor = loc_config.get(&36687)?;
        let name = trapdoor.name.as_ref()?;
        assert_eq!(name, "Trapdoor", "{:?}", trapdoor);
        Ok(())
    }

    #[test]
    fn check_paramtable() -> CacheResult<()> {
        use crate::structures::paramtable::Param;

        let loc_config = LocationConfig::dump_all()?;
        let bookcase = loc_config.get(&118445)?;
        let paramtable = bookcase.params.as_ref()?;
        let value = &paramtable.params[&8178];
        assert_eq!(*value, Param::Integer(50923), "{:?}", paramtable);
        Ok(())
    }
}
