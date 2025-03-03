use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::Write,
};

use ::error::Context;
use bytes::{Buf, Bytes};
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rayon::iter::{ParallelBridge, ParallelIterator};
use rs3cache_backend::{
    buf::{BufExtra, JString, NotExhausted, OpcodeNotImplemented, ReadError, WithInfo},
    error::{self, CacheResult},
    index::CacheIndex,
};
use serde::Serialize;

#[cfg(any(feature = "rs3", feature = "osrs"))]
use crate::definitions::indextype::IndexType;
use crate::structures::paramtable::ParamTable;

/// Describes the properties of a given [`Location`](crate::definitions::locations::Location).

#[cfg_attr(feature = "pyo3", pyclass(frozen, get_all))]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct LocationConfig {
    /// Its id.
    pub id: u32,
    /// A mapping of possible types to models.
    #[serde(flatten)]
    pub models: Option<Models>,
    /// Its name, if present.
    pub name: Option<JString<Bytes>>,
    /// Actually, this field is still in use for a little while after thw switch to dat2.
    #[cfg(feature = "legacy")]
    pub description: Option<JString<Bytes>>,
    #[cfg(any(feature = "osrs", feature = "legacy"))]
    #[serde(flatten)]
    pub models_2: Option<Models2>,
    /// Its west-east dimension, defaulting to 1 if not present.
    ///
    /// Code using this value must account for the location's rotation.
    pub dim_x: Option<u8>,
    /// Its south-north dimension, defaulting to 1 if not present.
    ///
    /// Code using this value must account for the location's rotation.
    pub dim_y: Option<u8>,
    #[cfg(feature = "2008_3_shim")]
    pub unknown_16: Option<bool>,
    pub unknown_17: Option<bool>,
    pub is_transparent: Option<bool>,
    /// Flag for whether this object has a red rather than a white line on the map.
    pub unknown_19: Option<u8>,
    pub unknown_21: Option<bool>,
    pub unknown_22: Option<bool>,
    pub occludes_1: Option<bool>,
    pub unknown_24: Option<u32>,
    pub unknown_25: Option<bool>,
    pub unknown_27: Option<bool>,
    pub unknown_28: Option<u8>,
    pub ambient: Option<i8>,
    /// What rightclick options this location has, if any.
    pub actions: Option<[Option<JString<Bytes>>; 5]>,
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
    pub breakroutefinding: Option<bool>,
    pub unknown_75: Option<u8>,
    /// This location can have different appearances depending on a player's varp/varbits.
    pub morphs_1: Option<LocationMorphTable>,
    pub unknown_78: Option<Unknown78>,
    pub unknown_79: Option<Unknown79>,
    pub unknown_81: Option<u8>,
    #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
    pub unknown_82: Option<bool>,
    #[cfg(all(feature = "osrs", not(feature = "2008_3_shim")))]
    pub maparea_id: Option<u16>,
    pub unknown_88: Option<bool>,
    pub unknown_89: Option<bool>,
    #[cfg(any(feature = "2008_3_shim", feature = "osrs"))]
    pub unknown_90: Option<bool>,
    pub is_members: Option<bool>,
    /// This location can have different appearances depending on a players varbits,
    /// like the [morphs_1](LocationConfig::morphs_1) field, but with a default value.
    pub morphs_2: Option<ExtendedLocationMorphTable>,
    pub unknown_93: Option<u16>,
    pub unknown_94: Option<bool>,
    #[cfg(any(feature = "rs3", feature = "2010_3_shim"))]
    pub unknown_95: Option<u16>,
    #[cfg(all(feature = "2008_3_shim", not(feature = "2010_3_shim")))]
    pub unknown_95: Option<bool>,
    #[cfg(feature = "2008_3_shim")]
    pub unknown_96: Option<bool>,
    pub unknown_97: Option<bool>,
    pub unknown_98: Option<bool>,
    pub unknown_99: Option<()>,
    pub unknown_101: Option<u8>,
    /// Reference to a [`MapScene`](super::mapscenes::MapScene) that is drawn on the map.
    ///
    /// This works differently between rs3 and osrs
    pub mapscene: Option<u16>,
    pub occludes_2: Option<bool>,
    pub unknown_104: Option<u8>,
    pub headmodels: Option<HeadModels>,
    pub mapfunction: Option<u16>,
    pub unknown_array: Option<[Option<u8>; 5]>,
    pub member_actions: Option<[Option<JString<Bytes>>; 5]>,
    pub unknown_159: Option<u8>,
    pub unknown_160: Option<Unknown160>,
    pub unknown_162: Option<i32>,
    pub unknown_163: Option<Unknown163>,
    pub unknown_164: Option<u16>,
    pub unknown_165: Option<u16>,
    pub unknown_166: Option<u16>,
    pub unknown_167: Option<u16>,
    #[cfg(feature = "2010_3_shim")]
    pub unknown_168: Option<bool>,
    #[cfg(feature = "2010_3_shim")]
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
    pub unknown_203: Option<bool>,
    pub unknown_204: Option<Vec<Unknown204>>,
    #[serde(flatten)]
    pub params: Option<ParamTable>,
}

impl LocationConfig {
    /// Returns a mapping of all [location configurations](LocationConfig)
    #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        let index = IndexType::LOC_CONFIG;

        let archives = CacheIndex::new(index, config.input.clone())?.into_iter();
        let locations = archives
            .map(Result::unwrap)
            .flat_map(|archive| {
                let archive_id = archive.archive_id();
                archive
                    .take_files()
                    .into_iter()
                    .map(move |(file_id, file)| (archive_id << 8 | file_id, file))
            })
            .map(|(id, file)| Self::deserialize(id, file).map(|item| (id, item)))
            .collect::<Result<BTreeMap<u32, Self>, ReadError>>()
            .context(error::Read { what: "location configs" })?;
        Ok(locations)
    }

    #[cfg(all(feature = "osrs", not(feature = "2008_3_shim"), not(feature = "legacy")))]
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        use crate::definitions::indextype::ConfigType;

        let locations = CacheIndex::new(IndexType::CONFIG, config.input.clone())?
            .archive(ConfigType::LOC_CONFIG)?
            .take_files()
            .into_iter()
            .map(|(id, file)| Self::deserialize(id, file).map(|item| (id, item)))
            .collect::<Result<BTreeMap<u32, Self>, ReadError>>()
            .context(error::Read { what: "location configs" })?;
        Ok(locations)
    }

    #[cfg(feature = "legacy")]
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        let cache = CacheIndex::new(0, config.input.clone()).unwrap();
        let archive = cache.archive(2).unwrap();
        let mut file = archive.file_named("loc.dat").unwrap();

        let _count = file.try_get_u16().unwrap();
        let mut offset_data = archive.file_named("loc.idx").unwrap();

        let mut locations = BTreeMap::new();

        let len = offset_data.try_get_u16().unwrap();
        for id in 0..len {
            let piece_len = offset_data.try_get_u16().context(error::Read {
                what: "location config offsets",
            })?;
            let data = file.split_to(piece_len as usize);
            let loc = LocationConfig::deserialize(id as u32, data).context(error::Read { what: "location configs" })?;
            locations.insert(id as u32, loc);
        }

        Ok(locations)
    }

    fn deserialize(id: u32, mut buffer: Bytes) -> Result<Self, ReadError> {
        let mut loc = Self { id, ..Default::default() };

        #[cfg(debug_assertions)]
        let mut opcodes = Vec::new();

        loop {
            let opcode = buffer.try_get_u8()?;
            let read: Result<(), ReadError> = try {
                match opcode {
                    0 => {
                        if buffer.has_remaining() {
                            return Err(NotExhausted::new(buffer));
                        } else {
                            break Ok(loc);
                        }
                    }
                    1 => loc.models = Some(Models::deserialize(&mut buffer)?),
                    2 => loc.name = Some(buffer.try_get_string()?),
                    // Actually, this field is still in use for a little while after the switch to dat2.
                    #[cfg(feature = "legacy")]
                    3 => loc.description = Some(buffer.try_get_string()?),
                    #[cfg(any(feature = "osrs", feature = "legacy"))]
                    5 => {
                        #[cfg(feature = "2010_1_shim")]
                        {
                            loc.models = Some(Models::deserialize(&mut buffer)?);
                        }

                        loc.models_2 = Some(Models2::deserialize(&mut buffer)?);
                    }

                    14 => loc.dim_x = Some(buffer.try_get_u8()?),
                    15 => loc.dim_y = Some(buffer.try_get_u8()?),
                    #[cfg(feature = "2008_3_shim")]
                    16 => loc.unknown_16 = Some(true),

                    17 => loc.unknown_17 = Some(false),
                    18 => loc.is_transparent = Some(true),
                    19 => loc.unknown_19 = Some(buffer.try_get_u8()?),
                    21 => loc.unknown_21 = Some(true),
                    22 => loc.unknown_22 = Some(true),
                    23 => loc.occludes_1 = Some(false),
                    24 => loc.unknown_24 = buffer.try_get_smart32()?,
                    #[cfg(feature = "legacy")]
                    25 => loc.unknown_25 = Some(true),
                    27 => loc.unknown_27 = Some(false),
                    28 => loc.unknown_28 = Some(buffer.try_get_u8()?),
                    29 => loc.ambient = Some(buffer.try_get_i8()?),
                    opcode @ 30..=34 => {
                        let actions = loc.actions.get_or_insert([None, None, None, None, None]);
                        actions[opcode as usize - 30] = Some(buffer.try_get_string()?);
                    }
                    39 => loc.contrast = Some(buffer.try_get_i8()?),
                    40 => loc.colour_replacements = Some(ColourReplacements::deserialize(&mut buffer)?),
                    41 => loc.textures = Some(Textures::deserialize(&mut buffer)?),

                    44 => loc.unknown_44 = Some(buffer.try_get_masked_index()?),
                    45 => loc.unknown_45 = Some(buffer.try_get_masked_index()?),
                    // changed at some point after 2015
                    // used to be mapscenes
                    // see https://discordapp.com/channels/177206626514632704/269673599554551808/872603876384178206
                    #[cfg(any(feature = "osrs", feature = "legacy"))]
                    60 => loc.mapfunction = Some(buffer.try_get_u16()?),

                    #[cfg(feature = "osrs")]
                    61 => loc.category = Some(buffer.try_get_u16()?),
                    62 => loc.mirror = Some(true),
                    64 => loc.model = Some(false),
                    65 => loc.scale_x = Some(buffer.try_get_u16()?),
                    66 => loc.scale_y = Some(buffer.try_get_u16()?),
                    67 => loc.scale_z = Some(buffer.try_get_u16()?),
                    #[cfg(any(feature = "osrs", feature = "legacy"))]
                    68 => loc.mapscene = Some(buffer.try_get_u16()?),
                    69 => loc.unknown_69 = Some(buffer.try_get_u8()?),
                    70 => loc.translate_x = Some(buffer.try_get_u16()?),
                    71 => loc.translate_y = Some(buffer.try_get_u16()?),
                    72 => loc.translate_z = Some(buffer.try_get_u16()?),
                    73 => loc.unknown_73 = Some(true),
                    74 => loc.breakroutefinding = Some(true),
                    75 => loc.unknown_75 = Some(buffer.try_get_u8()?),
                    77 => loc.morphs_1 = Some(LocationMorphTable::deserialize(&mut buffer)?),
                    78 => loc.unknown_78 = Some(Unknown78::deserialize(&mut buffer)?),
                    79 => loc.unknown_79 = Some(Unknown79::deserialize(&mut buffer)?),
                    81 => loc.unknown_81 = Some(buffer.try_get_u8()?),
                    #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
                    82 => loc.unknown_82 = Some(true),
                    #[cfg(all(feature = "osrs", not(feature = "2008_3_shim")))]
                    82 => loc.maparea_id = Some(buffer.try_get_u16()?),
                    88 => loc.unknown_88 = Some(false),
                    89 => loc.unknown_89 = Some(false),
                    #[cfg(any(feature = "2008_3_shim", feature = "osrs"))]
                    90 => loc.unknown_90 = Some(true),
                    91 => loc.is_members = Some(true),
                    92 => loc.morphs_2 = Some(ExtendedLocationMorphTable::deserialize(&mut buffer)?),
                    93 => loc.unknown_93 = Some(buffer.try_get_u16()?),
                    94 => loc.unknown_94 = Some(true),
                    #[cfg(all(feature = "2008_3_shim", not(feature = "2010_3_shim")))]
                    95 => loc.unknown_95 = Some(true),
                    #[cfg(any(feature = "rs3", feature = "2010_3_shim"))]
                    95 => loc.unknown_95 = Some(buffer.try_get_u16()?),
                    #[cfg(feature = "2008_3_shim")]
                    96 => loc.unknown_96 = Some(true),
                    97 => loc.unknown_97 = Some(true),
                    98 => loc.unknown_98 = Some(true),
                    #[cfg(feature = "2009_1_shim")]
                    opcode @ 99..=100 => {
                        let cursors = loc.cursors.get_or_insert([None, None, None, None, None, None]);
                        buffer.try_get_u8()?;
                        cursors[opcode as usize - 99] = Some(buffer.try_get_u16()?);
                    }
                    #[cfg(any(feature = "rs3", feature = "2008_3_shim"))]
                    102 => loc.mapscene = Some(buffer.try_get_u16()?),
                    103 => loc.occludes_2 = Some(false),
                    104 => loc.unknown_104 = Some(buffer.try_get_u8()?),
                    106 => loc.headmodels = Some(HeadModels::deserialize(&mut buffer)?),
                    #[cfg(any(feature = "rs3", feature = "2009_1_shim"))]
                    107 => loc.mapfunction = Some(buffer.try_get_u16()?),
                    #[cfg(not(feature = "2010_1_shim"))]
                    opcode @ 136..=140 => {
                        let actions = loc.unknown_array.get_or_insert([None, None, None, None, None]);
                        actions[opcode as usize - 136] = Some(buffer.try_get_u8()?);
                    }
                    opcode @ 150..=154 => {
                        let actions = loc.member_actions.get_or_insert([None, None, None, None, None]);
                        actions[opcode as usize - 150] = Some(buffer.try_get_string()?);
                    }

                    159 => loc.unknown_159 = Some(buffer.try_get_u8()?),
                    160 => loc.unknown_160 = Some(Unknown160::deserialize(&mut buffer)?),
                    162 => loc.unknown_162 = Some(buffer.try_get_i32()?),
                    163 => loc.unknown_163 = Some(Unknown163::deserialize(&mut buffer)?),
                    164 => loc.unknown_164 = Some(buffer.try_get_u16()?),
                    165 => loc.unknown_166 = Some(buffer.try_get_u16()?),
                    167 => loc.unknown_167 = Some(buffer.try_get_u16()?),
                    #[cfg(feature = "2010_3_shim")]
                    168 => loc.unknown_168 = Some(true),
                    #[cfg(feature = "2010_3_shim")]
                    169 => loc.unknown_169 = Some(true),
                    170 => loc.unknown_170 = Some(buffer.try_get_unsigned_smart()?),
                    171 => loc.unknown_171 = Some(buffer.try_get_unsigned_smart()?),
                    173 => loc.unknown_173 = Some(Unknown173::deserialize(&mut buffer)?),
                    177 => loc.unknown_177 = Some(true),
                    178 => loc.unknown_178 = Some(buffer.try_get_u8()?),
                    186 => loc.unknown_186 = Some(buffer.try_get_u8()?),
                    188 => loc.unknown_188 = Some(true),
                    189 => loc.unknown_189 = Some(true),
                    // august 2012
                    opcode @ 190..=195 => {
                        let actions = loc.cursors.get_or_insert([None, None, None, None, None, None]);
                        actions[opcode as usize - 190] = Some(buffer.try_get_u16()?);
                    }
                    196 => loc.unknown_196 = Some(buffer.try_get_u8()?),
                    197 => loc.unknown_196 = Some(buffer.try_get_u8()?),
                    198 => loc.unknown_198 = Some(true),
                    199 => loc.unknown_199 = Some(true),
                    201 => loc.unknown_201 = Some(Unknown201::deserialize(&mut buffer)?),
                    202 => loc.unknown_202 = Some(buffer.try_get_unsigned_smart()?),
                    203 => loc.unknown_203 = Some(true),
                    204 => {
                        let count = buffer.try_get_unsigned_smart()? as usize;
                        let out = std::iter::repeat_with(|| Unknown204::deserialize(&mut buffer))
                            .take(count)
                            .collect::<Result<_, _>>()?;

                        loc.unknown_204 = Some(out)
                    }
                    249 => loc.params = Some(ParamTable::deserialize(&mut buffer)),
                    opcode => {
                        println!("{loc}");
                        do yeet OpcodeNotImplemented::new(opcode)
                    }
                }
            };
            match read {
                Ok(()) => {
                    #[cfg(debug_assertions)]
                    opcodes.push(opcode);
                }
                Err(e) => {
                    return Err(Box::new(e)).context(WithInfo {
                        #[cfg(debug_assertions)]
                        opcodes,
                        buffer,
                        #[cfg(debug_assertions)]
                        thing: loc.to_string(),
                    })
                }
            }
        }
    }
}

impl Display for LocationConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

/// Defines the structs used as fields of [`LocationConfig`],
pub mod location_config_fields {

    use std::{collections::BTreeMap, iter};

    use bytes::Bytes;
    #[cfg(feature = "pyo3")]
    use pyo3::prelude::*;
    use rs3cache_backend::buf::{BufExtra, ReadError};
    use serde::Serialize;

    use crate::types::variables::{Varbit, Varp, VarpOrVarbit};

    #[cfg(any(feature = "rs3", feature = "2011_11_shim"))]
    type IdType = u32;

    #[cfg(all(any(feature = "osrs", feature = "legacy"), not(feature = "2011_11_shim")))]
    type IdType = u16;

    /// Contains an array of possible ids this location can morph into, controlled by either a varbit or varp.

    #[cfg_attr(feature = "pyo3", pyclass(frozen, get_all))]
    #[derive(Serialize, Debug, Clone)]
    pub struct LocationMorphTable {
        #[serde(flatten)]
        pub var: VarpOrVarbit,
        pub ids: Vec<Option<IdType>>,
    }

    impl LocationMorphTable {
        /// Constructor for [`LocationMorphTable`]
        #[cfg(any(feature = "rs3", feature = "2011_11_shim"))]
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let varbit = Varbit::new(buffer.try_get_u16()?);
            let varp = Varp::new(buffer.try_get_u16()?);
            let var = VarpOrVarbit::new(varp, varbit);

            let count = if cfg!(feature = "2011_11_shim") {
                buffer.try_get_u8()? as usize
            } else {
                buffer.try_get_unsigned_smart()? as usize
            };

            let ids = iter::repeat_with(|| buffer.try_get_smart32())
                .take(count + 1)
                .collect::<Result<_, ReadError>>()?;

            Ok(Self { var, ids })
        }

        #[cfg(all(any(feature = "osrs", feature = "legacy"), not(feature = "2011_11_shim")))]
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let varbit = Varbit::new(buffer.try_get_u16()?);
            let varp = Varp::new(buffer.try_get_u16()?);
            let var = VarpOrVarbit::new(varp, varbit);

            let count = buffer.try_get_u8()? as usize;

            let ids = iter::repeat_with(|| {
                buffer.try_get_u16().map(|id| match id {
                    0xFFFF => None,
                    id => Some(id),
                })
            })
            .take(count + 1)
            .collect::<Result<_, ReadError>>()?;

            Ok(Self { var, ids })
        }
    }

    /// Like [`LocationMorphTable`], but with a default value.

    #[cfg_attr(feature = "pyo3", pyclass(frozen, get_all))]
    #[derive(Serialize, Debug, Clone)]
    pub struct ExtendedLocationMorphTable {
        #[serde(flatten)]
        pub var: VarpOrVarbit,

        /// The possible ids this [`LocationConfig`](super::LocationConfig) can be.
        pub ids: Vec<Option<IdType>>,

        /// This [`LocationConfig`](super::LocationConfig)'s default id.
        pub default: Option<IdType>,
    }

    impl ExtendedLocationMorphTable {
        /// Constructor for [`ExtendedLocationMorphTable`]
        #[cfg(any(feature = "rs3", feature = "2011_11_shim"))]
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let varbit = Varbit::new(buffer.try_get_u16()?);
            let varp = Varp::new(buffer.try_get_u16()?);

            let var = VarpOrVarbit::new(varp, varbit);

            let default = if cfg!(all(feature = "2011_11_shim", not(feature = "2013_shim"))) {
                Some(buffer.try_get_u16()? as u32)
            } else {
                buffer.try_get_smart32()?
            };

            let count = if cfg!(feature = "2011_11_shim") {
                buffer.try_get_u8()? as usize
            } else {
                buffer.try_get_unsigned_smart()? as usize
            };

            let ids = iter::repeat_with(|| buffer.try_get_smart32())
                .take(count + 1)
                .collect::<Result<_, ReadError>>()?;
            Ok(Self { var, ids, default })
        }

        #[cfg(all(any(feature = "osrs", feature = "legacy"), not(feature = "2011_11_shim")))]
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let varbit = Varbit::new(buffer.try_get_u16()?);
            let varp = Varp::new(buffer.try_get_u16()?);

            let var = VarpOrVarbit::new(varp, varbit);

            let default = match buffer.try_get_u16()? {
                0xFFFF => None,
                id => Some(id),
            };

            let count = buffer.try_get_u8()? as usize;
            let ids = iter::repeat_with(|| {
                buffer.try_get_u16().map(|id| match id {
                    0xFFFF => None,
                    id => Some(id),
                })
            })
            .take(count + 1)
            .collect::<Result<_, ReadError>>()?;

            Ok(Self { var, ids, default })
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct ColourReplacements {
        pub colours: Vec<(u16, u16)>,
    }

    impl ColourReplacements {
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let count = buffer.try_get_u8()? as usize;
            let colours = iter::repeat_with(|| try { (buffer.try_get_u16()?, buffer.try_get_u16()?) })
                .take(count)
                .collect::<Result<Vec<(u16, u16)>, ReadError>>()?;
            Ok(Self { colours })
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct Models {
        #[cfg(any(feature = "rs3", feature = "2010_1_shim"))]
        pub models: BTreeMap<i8, Vec<Option<u32>>>,
        #[cfg(all(any(feature = "osrs", feature = "legacy"), not(feature = "2010_1_shim")))]
        pub models: Vec<(u8, u16)>,
    }

    impl Models {
        #[cfg(any(feature = "rs3", feature = "2010_1_shim"))]
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            fn sub_deserialize(buffer: &mut Bytes) -> Result<(i8, Vec<Option<u32>>), ReadError> {
                let ty = buffer.try_get_i8()?;
                let count = buffer.try_get_u8()? as usize;
                let values = iter::repeat_with(|| try {
                    if cfg!(all(feature = "2010_1_shim", not(feature = "2011_11_shim"))) {
                        Some(buffer.try_get_u16()? as u32)
                    } else {
                        buffer.try_get_smart32()?
                    }
                })
                .take(count)
                .collect::<Result<_, ReadError>>()?;
                Ok((ty, values))
            }

            let count = buffer.try_get_u8()? as usize;

            let models = iter::repeat_with(|| sub_deserialize(buffer))
                .take(count)
                .collect::<Result<BTreeMap<_, _>, ReadError>>()?;
            Ok(Models { models })
        }

        #[cfg(all(any(feature = "osrs", feature = "legacy"), not(feature = "2010_1_shim")))]
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let count = buffer.try_get_u8()? as usize;

            let models = iter::repeat_with(|| try {
                let model = buffer.try_get_u16()?;
                let r#type = buffer.try_get_u8()?;
                (r#type, model)
            })
            .take(count)
            .collect::<Result<_, ReadError>>()?;
            Ok(Models { models })
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct Models2 {
        #[cfg(all(any(feature = "osrs", feature = "legacy"), not(feature = "2010_1_shim")))]
        pub models_2: Vec<u16>,
        #[cfg(any(feature = "rs3", feature = "2010_1_shim"))]
        pub models_2: BTreeMap<u8, u32>,
    }

    impl Models2 {
        #[cfg(all(any(feature = "osrs", feature = "legacy"), not(feature = "2010_1_shim")))]
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let count = buffer.try_get_u8()? as usize;

            let models_2 = iter::repeat_with(|| buffer.try_get_u16()).take(count).collect::<Result<_, ReadError>>()?;
            Ok(Self { models_2 })
        }

        #[cfg(any(feature = "rs3", feature = "2010_1_shim"))]
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let count = buffer.try_get_u8()? as usize;

            let models_2 = iter::repeat_with(|| try {
                let r#type = buffer.try_get_u8()?;
                let subcount = buffer.try_get_u8()?;

                let model = if cfg!(feature = "2011_11_shim") {
                    buffer.try_get_smart32()?.unwrap()
                } else {
                    buffer.try_get_u16()? as u32
                };

                for _ in 1..subcount {
                    buffer.try_get_u16()?;
                }

                (r#type, model)
            })
            .take(count)
            .collect::<Result<_, ReadError>>()?;
            Ok(Self { models_2 })
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct Textures {
        pub textures: BTreeMap<u16, u16>,
    }

    impl Textures {
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let count = buffer.try_get_u8()? as usize;
            let textures = iter::repeat_with(|| try { (buffer.try_get_u16()?, buffer.try_get_u16()?) })
                .take(count)
                .collect::<Result<BTreeMap<_, _>, ReadError>>()?;
            Ok(Textures { textures })
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown79 {
        pub unknown_1: u16,

        pub unknown_2: u16,

        pub unknown_3: u8,

        pub values: Vec<u16>,
    }

    impl Unknown79 {
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let unknown_1 = buffer.try_get_u16()?;
            let unknown_2 = buffer.try_get_u16()?;
            let unknown_3 = buffer.try_get_u8()?;
            if cfg!(feature = "osrs") {
                //FIXME: Post rev 220
                let _sound_retain = buffer.try_get_u8()?;
            }

            let count = buffer.try_get_u8()? as usize;

            let values = iter::repeat_with(|| buffer.try_get_u16()).take(count).collect::<Result<_, ReadError>>()?;

            Ok(Unknown79 {
                unknown_1,
                unknown_2,
                unknown_3,
                values,
            })
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct Unknown173 {
        pub unknown_1: u16,

        pub unknown_2: u16,
    }

    impl Unknown173 {
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let unknown_1 = buffer.try_get_u16()?;
            let unknown_2 = buffer.try_get_u16()?;

            Ok(Unknown173 { unknown_1, unknown_2 })
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct Unknown163 {
        pub unknown_1: i8,

        pub unknown_2: i8,

        pub unknown_3: i8,

        pub unknown_4: i8,
    }

    impl Unknown163 {
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let unknown_1 = buffer.try_get_i8()?;
            let unknown_2 = buffer.try_get_i8()?;
            let unknown_3 = buffer.try_get_i8()?;
            let unknown_4 = buffer.try_get_i8()?;

            Ok(Unknown163 {
                unknown_1,
                unknown_2,
                unknown_3,
                unknown_4,
            })
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone, Copy)]
    pub struct Unknown78 {
        pub unknown_1: u16,

        pub unknown_2: u8,
    }

    impl Unknown78 {
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let unknown_1 = buffer.try_get_u16()?;
            let unknown_2 = buffer.try_get_u8()?;
            if cfg!(feature = "osrs") {
                // FIXME: Post rev 220
                let _sound_retain = buffer.try_get_u8()?;
            }

            Ok(Self { unknown_1, unknown_2 })
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown160 {
        pub values: Vec<u16>,
    }

    impl Unknown160 {
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let count = buffer.try_get_u8()? as usize;
            let values = iter::repeat_with(|| buffer.try_get_u16()).take(count).collect::<Result<_, ReadError>>()?;
            Ok(Self { values })
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
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
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let unknown_1 = buffer.try_get_unsigned_smart()?;
            let unknown_2 = buffer.try_get_unsigned_smart()?;
            let unknown_3 = buffer.try_get_unsigned_smart()?;
            let unknown_4 = buffer.try_get_unsigned_smart()?;
            let unknown_5 = buffer.try_get_unsigned_smart()?;
            let unknown_6 = buffer.try_get_unsigned_smart()?;

            Ok(Unknown201 {
                unknown_1,
                unknown_2,
                unknown_3,
                unknown_4,
                unknown_5,
                unknown_6,
            })
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct HeadModels {
        pub headmodels: Vec<(Option<u32>, u8)>,
    }

    impl HeadModels {
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let count = buffer.try_get_u8()? as usize;
            let headmodels = iter::repeat_with(|| try { (buffer.try_get_smart32()?, buffer.try_get_u8()?) })
                .take(count)
                .collect::<Result<_, ReadError>>()?;
            Ok(HeadModels { headmodels })
        }
    }

    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct Unknown204 {
        pub id: u16,
        pub some_bool: bool,
        pub vec_1: [u32; 3],
        pub vec_2: [u32; 3],
    }

    impl Unknown204 {
        pub fn deserialize(buffer: &mut Bytes) -> Result<Self, ReadError> {
            let id = buffer.try_get_u16()?;
            let some_bool = buffer.try_get_u8()? == 1;
            let vec_1 = [buffer.try_get_u32()?, buffer.try_get_u32()?, buffer.try_get_u32()?];
            let vec_2 = [buffer.try_get_u32()?, buffer.try_get_u32()?, buffer.try_get_u32()?];

            Ok(Self { id, some_bool, vec_1, vec_2 })
        }
    }
}

use location_config_fields::*;

/// Save the location configs as `location_configs.json`. Exposed as `--dump location_configs`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output).with_context(|| error::Io { path: config.output.clone() })?;
    let loc_configs = LocationConfig::dump_all(config)?.into_values().collect::<Vec<_>>();
    let path = path!(config.output / "location_configs.json");
    let mut file = File::create(&path).with_context(|| error::Io { path: path.clone() })?;
    let data = serde_json::to_string_pretty(&loc_configs).unwrap();
    file.write_all(data.as_bytes()).context(error::Io { path })?;

    Ok(())
}

///Save the location configs as individual `json` files.
pub fn export_each(config: &crate::cli::Config) -> CacheResult<()> {
    let folder = path!(&config.output / "location_configs");
    fs::create_dir_all(&folder).with_context(|| error::Io { path: folder.clone() })?;

    let configs = LocationConfig::dump_all(config)?;
    configs.into_iter().par_bridge().try_for_each(|(id, location_config)| {
        let path = path!(&folder / format!("{id}.json"));
        let mut file = File::create(&path).with_context(|| error::Io { path: path.clone() })?;
        let data = serde_json::to_string_pretty(&location_config).unwrap();
        file.write_all(data.as_bytes()).context(error::Io { path })?;
        Ok(())
    })
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl LocationConfig {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("LocationConfig({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("LocationConfig({})", serde_json::to_string(self).unwrap()))
    }
}

#[cfg(all(test, feature = "legacy"))]
mod legacy {

    use rs3cache_backend::index::CacheIndex;

    use super::*;
    use crate::cli::Config;

    #[test]
    fn decode_locations() {
        let config = Config::env();
        let location_count = 7389;
        let first_section = 0..45;

        let cache = CacheIndex::new(0, config.input).unwrap();
        let archive = cache.archive(2).unwrap();
        let mut file = archive.file_named("loc.dat").unwrap();

        let count = file.try_get_u16().unwrap();

        assert_eq!(
            &file[first_section],
            b"\x1eSearch\n\x05\x01\x08]\x02Crate\n\x03I wonder what's inside.\n\0"
        );
        assert_eq!(count, location_count);

        let mut offset_data = archive.file_named("loc.idx").unwrap();

        let len = offset_data.try_get_u16().unwrap();
        for id in 0..len {
            let piece_len = offset_data.try_get_u16().unwrap();
            let data = file.split_to(piece_len as usize);
            let _loc = LocationConfig::deserialize(id as u32, data).unwrap();
            //println!("{}", loc);
        }
        assert_eq!(offset_data, &[].as_slice());
        assert_eq!(file, &[].as_slice());
        //panic!();
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
        let loc = loc_config.get(&3263).expect("Swamp not present");
        let name = loc.name.as_ref().expect("Swamp has no name");

        assert_eq!(name, "Swamp", "{loc:?}");

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
        assert_eq!(*value, Param::Integer(50923), "{paramtable:?}");
        Ok(())
    }
}
