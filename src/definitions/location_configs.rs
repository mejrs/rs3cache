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
    cache::{buf::Buffer, index::CacheIndex, indextype::IndexType},
    structures::paramtable::ParamTable,
    utils::{error::CacheResult, par::ParApply},
};

/// Describes the properties of a given [`Location`](crate::definitions::locations::Location).
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[skip_serializing_none]
#[derive(Serialize, Debug, Default)]
pub struct LocationConfig {
    /// Its id.
    pub id: u32,
    /// A mapping of possible types to models.
    #[serde(flatten)]
    pub models: Option<Models>,
    /// Its name, if present.
    pub name: Option<String>,
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
    pub unknown_82: Option<bool>,
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

/// Defines the structs used as fields of [`LocationConfig`],
pub mod location_config_fields {
    #![allow(missing_docs)]
    use std::{collections::HashMap, iter};

    #[cfg(feature = "pyo3")]
    use pyo3::prelude::*;
    use serde::Serialize;

    use crate::{
        cache::buf::Buffer,
        types::variables::{Varbit, Varp, VarpOrVarbit},
    };
    /// Contains an array of possible ids this location can morph into, controlled by either a varbit or varp.
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct LocationMorphTable {
        #[serde(flatten)]
        pub var: VarpOrVarbit,

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
    #[cfg_attr(feature = "pyo3", pyclass)]
    #[allow(missing_docs)]
    #[derive(Serialize, Debug, Clone)]
    pub struct ExtendedLocationMorphTable {
        #[serde(flatten)]
        pub var: VarpOrVarbit,

        /// The possible ids this [`LocationConfig`](super::LocationConfig) can be.
        pub ids: Vec<Option<u32>>,

        /// This [`LocationConfig`](super::LocationConfig)'s default id.
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

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct ColourReplacements {
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

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct Models {
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

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct Textures {
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

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown79 {
        pub unknown_1: u16,

        pub unknown_2: u16,

        pub unknown_3: u8,

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

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct Unknown173 {
        pub unknown_1: u16,

        pub unknown_2: u16,
    }

    impl Unknown173 {
        pub fn deserialize(buffer: &mut Buffer) -> Unknown173 {
            let unknown_1 = buffer.read_unsigned_short();
            let unknown_2 = buffer.read_unsigned_short();

            Unknown173 { unknown_1, unknown_2 }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct Unknown163 {
        pub unknown_1: i8,

        pub unknown_2: i8,

        pub unknown_3: i8,

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

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct Unknown78 {
        pub unknown_1: u16,

        pub unknown_2: u8,
    }

    impl Unknown78 {
        pub fn deserialize(buffer: &mut Buffer) -> Unknown78 {
            let unknown_1 = buffer.read_unsigned_short();
            let unknown_2 = buffer.read_unsigned_byte();

            Unknown78 { unknown_1, unknown_2 }
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown160 {
        pub values: Vec<u16>,
    }

    impl Unknown160 {
        pub fn deserialize(buffer: &mut Buffer) -> Unknown160 {
            let count = buffer.read_unsigned_byte() as usize;
            let values = iter::repeat_with(|| buffer.read_unsigned_short()).take(count).collect::<Vec<_>>();
            Unknown160 { values }
        }
    }

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

    #[cfg_attr(feature = "pyo3", pyclass)]
    #[derive(Serialize, Debug, Clone)]
    pub struct HeadModels {
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

#[cfg(feature = "pyo3")]
#[pymethods]
impl LocationConfig {
    #[getter]
    fn id(&self) -> PyResult<u32> {
        Ok(self.id)
    }
    #[getter]
    fn models(&self) -> PyResult<Option<Models>> {
        Ok(self.models.clone())
    }
    #[getter]
    fn name(&self) -> PyResult<Option<String>> {
        Ok(self.name.clone())
    }
    #[getter]
    fn dim_x(&self) -> PyResult<Option<u8>> {
        Ok(self.dim_x)
    }
    #[getter]
    fn dim_y(&self) -> PyResult<Option<u8>> {
        Ok(self.dim_y)
    }
    #[getter]
    fn unknown_17(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_17)
    }
    #[getter]
    fn is_transparent(&self) -> PyResult<Option<bool>> {
        Ok(self.is_transparent)
    }
    #[getter]
    fn unknown_19(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_19)
    }
    #[getter]
    fn unknown_21(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_21)
    }
    #[getter]
    fn unknown_22(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_22)
    }
    #[getter]
    fn occludes_1(&self) -> PyResult<Option<bool>> {
        Ok(self.occludes_1)
    }
    #[getter]
    fn unknown_24(&self) -> PyResult<Option<u32>> {
        Ok(self.unknown_24)
    }
    #[getter]
    fn unknown_27(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_27)
    }
    #[getter]
    fn unknown_28(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_28)
    }
    #[getter]
    fn ambient(&self) -> PyResult<Option<i8>> {
        Ok(self.ambient)
    }
    #[getter]
    fn actions(&self) -> PyResult<Option<[Option<String>; 5]>> {
        Ok(self.actions.clone())
    }
    #[getter]
    fn contrast(&self) -> PyResult<Option<i8>> {
        Ok(self.contrast)
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
    fn recolour_palette(&self) -> PyResult<Option<Vec<(u16, u16)>>> {
        Ok(self.recolour_palette.clone())
    }
    #[getter]
    fn unknown_44(&self) -> PyResult<Option<u16>> {
        Ok(self.unknown_44)
    }
    #[getter]
    fn unknown_45(&self) -> PyResult<Option<u16>> {
        Ok(self.unknown_45)
    }
    #[getter]
    fn mirror(&self) -> PyResult<Option<bool>> {
        Ok(self.mirror)
    }
    #[getter]
    fn model(&self) -> PyResult<Option<bool>> {
        Ok(self.model)
    }
    #[getter]
    fn scale_x(&self) -> PyResult<Option<u16>> {
        Ok(self.scale_x)
    }
    #[getter]
    fn scale_y(&self) -> PyResult<Option<u16>> {
        Ok(self.scale_y)
    }
    #[getter]
    fn scale_z(&self) -> PyResult<Option<u16>> {
        Ok(self.scale_z)
    }
    #[getter]
    fn unknown_69(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_69)
    }
    #[getter]
    fn translate_x(&self) -> PyResult<Option<u16>> {
        Ok(self.translate_x)
    }
    #[getter]
    fn translate_y(&self) -> PyResult<Option<u16>> {
        Ok(self.translate_y)
    }
    #[getter]
    fn translate_z(&self) -> PyResult<Option<u16>> {
        Ok(self.translate_z)
    }
    #[getter]
    fn unknown_73(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_73)
    }
    #[getter]
    fn blocks_ranged(&self) -> PyResult<Option<bool>> {
        Ok(self.blocks_ranged)
    }
    #[getter]
    fn unknown_75(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_75)
    }
    #[getter]
    fn morphs_1(&self) -> PyResult<Option<LocationMorphTable>> {
        Ok(self.morphs_1.clone())
    }
    #[getter]
    fn unknown_78(&self) -> PyResult<Option<Unknown78>> {
        Ok(self.unknown_78)
    }
    #[getter]
    fn unknown_79(&self) -> PyResult<Option<Unknown79>> {
        Ok(self.unknown_79.clone())
    }
    #[getter]
    fn unknown_81(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_81)
    }
    #[getter]
    fn unknown_82(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_82)
    }
    #[getter]
    fn unknown_88(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_88)
    }
    #[getter]
    fn unknown_89(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_89)
    }
    #[getter]
    fn is_members(&self) -> PyResult<Option<bool>> {
        Ok(self.is_members)
    }
    #[getter]
    fn morphs_2(&self) -> PyResult<Option<ExtendedLocationMorphTable>> {
        Ok(self.morphs_2.clone())
    }
    #[getter]
    fn unknown_93(&self) -> PyResult<Option<u16>> {
        Ok(self.unknown_93)
    }
    #[getter]
    fn unknown_94(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_94)
    }
    #[getter]
    fn unknown_95(&self) -> PyResult<Option<u16>> {
        Ok(self.unknown_95)
    }
    #[getter]
    fn unknown_96(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_96)
    }
    #[getter]
    fn unknown_97(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_97)
    }
    #[getter]
    fn unknown_98(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_98)
    }
    #[getter]
    fn unknown_99(&self) -> PyResult<Option<()>> {
        Ok(self.unknown_99)
    }
    #[getter]
    fn unknown_101(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_101)
    }
    #[getter]
    fn mapscene(&self) -> PyResult<Option<u16>> {
        Ok(self.mapscene)
    }
    #[getter]
    fn occludes_2(&self) -> PyResult<Option<bool>> {
        Ok(self.occludes_2)
    }
    #[getter]
    fn unknown_104(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_104)
    }
    #[getter]
    fn headmodels(&self) -> PyResult<Option<HeadModels>> {
        Ok(self.headmodels.clone())
    }
    #[getter]
    fn mapfunction(&self) -> PyResult<Option<u16>> {
        Ok(self.mapfunction)
    }
    #[getter]
    fn member_actions(&self) -> PyResult<Option<[Option<String>; 5]>> {
        Ok(self.member_actions.clone())
    }
    #[getter]
    fn unknown_160(&self) -> PyResult<Option<Unknown160>> {
        Ok(self.unknown_160.clone())
    }
    #[getter]
    fn unknown_162(&self) -> PyResult<Option<i32>> {
        Ok(self.unknown_162)
    }
    #[getter]
    fn unknown_163(&self) -> PyResult<Option<Unknown163>> {
        Ok(self.unknown_163)
    }
    #[getter]
    fn unknown_164(&self) -> PyResult<Option<u16>> {
        Ok(self.unknown_164)
    }
    #[getter]
    fn unknown_165(&self) -> PyResult<Option<u16>> {
        Ok(self.unknown_165)
    }
    #[getter]
    fn unknown_166(&self) -> PyResult<Option<u16>> {
        Ok(self.unknown_166)
    }
    #[getter]
    fn unknown_167(&self) -> PyResult<Option<u16>> {
        Ok(self.unknown_167)
    }
    #[getter]
    fn unknown_168(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_168)
    }
    #[getter]
    fn unknown_169(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_169)
    }
    #[getter]
    fn unknown_170(&self) -> PyResult<Option<u16>> {
        Ok(self.unknown_170)
    }
    #[getter]
    fn unknown_171(&self) -> PyResult<Option<u16>> {
        Ok(self.unknown_171)
    }
    #[getter]
    fn unknown_173(&self) -> PyResult<Option<Unknown173>> {
        Ok(self.unknown_173)
    }
    #[getter]
    fn unknown_177(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_177)
    }
    #[getter]
    fn unknown_178(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_178)
    }
    #[getter]
    fn unknown_186(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_186)
    }
    #[getter]
    fn unknown_188(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_188)
    }
    #[getter]
    fn unknown_189(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_189)
    }
    #[getter]
    fn cursors(&self) -> PyResult<Option<[Option<u16>; 6]>> {
        Ok(self.cursors)
    }
    #[getter]
    fn unknown_196(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_196)
    }
    #[getter]
    fn unknown_197(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_197)
    }
    #[getter]
    fn unknown_198(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_198)
    }
    #[getter]
    fn unknown_199(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_199)
    }
    #[getter]
    fn unknown_200(&self) -> PyResult<Option<bool>> {
        Ok(self.unknown_200)
    }
    #[getter]
    fn unknown_201(&self) -> PyResult<Option<Unknown201>> {
        Ok(self.unknown_201)
    }
    #[getter]
    fn unknown_202(&self) -> PyResult<Option<u16>> {
        Ok(self.unknown_202)
    }
    #[getter]
    fn params(&self) -> PyResult<Option<ParamTable>> {
        Ok(self.params.clone())
    }
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
