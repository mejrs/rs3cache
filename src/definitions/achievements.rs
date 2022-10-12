//! Describes the properties of Achievements.
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    iter,
};

use bytes::{Buf, Bytes};
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::Serialize;

use crate::{
    cache::{buf::BufExtra, error::CacheResult, index::CacheIndex},
    definitions::indextype::IndexType,
};
/// Describes the properties of a given Achievement.
#[cfg_eval]
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
#[cfg_attr(feature = "pyo3", pyclass(frozen))]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct Achievement {
    /// Its id.
    pub id: u32,
    /// Its name, if present.
    pub name: Option<String>,
    pub description: Option<String>,
    pub ironman_description: Option<String>,
    pub category: Option<u16>,
    pub sprite_id: Option<u32>,
    pub points: Option<u8>,

    /// 331 - Following In The Footsteps - 7126
    ///
    /// 332 - Totem Pole Position - 6830
    ///
    /// 334 - From Ardougne with Love - 6830
    ///
    /// 1087 - Penguin Adoption Agency - 6790
    pub unknown_6: Option<u16>,
    pub reward: Option<String>,
    pub skill_requirements_1: Option<Vec<MaybeIronmanSkillRequirement>>,
    pub unknown_9: Option<Vec<VarbitRequirement10>>,
    pub unknown_10: Option<Vec<VarbitRequirement10>>,
    pub previous_achievements: Option<Vec<u32>>,
    pub skill_requirements_2: Option<Vec<SkillRequirement>>,
    pub unknown_13: Option<Vec<MultipleVarbitsRequirement>>,
    pub subreqs_14: Option<Vec<MultipleVarbitsRequirement>>,
    pub sub_achievements: Option<Vec<u32>>,
    pub sub_category: Option<u16>,

    /// 1: hidden (e.g. chained achievements)
    ///
    /// 2: extra hidden (e.g. 4k telos)
    pub hidden: Option<u8>,
    pub free_to_play: Option<bool>,
    pub quest_req_for_miniquests: Option<Vec<u32>>,
    pub required_quest_ids: Option<Vec<u32>>,
    pub reqs_23: Option<Vec<PackedVarbitRequirement>>,
    pub reqs_25: Option<Vec<PackedVarbitRequirement>>,
    pub unknown_27: Option<bool>,
    pub unknown_17: Option<bool>, //
    pub skill_req_count: Option<Vec<u8>>,

    /// 355 - Seven Colours in Their Hat - 1
    ///
    /// 573 - Totem Pole Position - 1
    ///
    /// 642 - 99 With a Flake - 1
    ///
    /// 831 - Always Be Prepared - 1
    pub unknown_29: Option<u8>,
    pub sub_req_count: Option<Vec<u16>>,
    pub unknown_31: Option<u8>,
    pub unknown_32: Option<[u8; 3]>,
    pub unknown_35: Option<bool>,
    pub unknown_37: Option<u8>,
    pub unknown_38: Option<u8>,
}

impl Achievement {
    /// Returns a mapping of all [`Achievement`]s.
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, Self>> {
        let archives = CacheIndex::new(IndexType::ACHIEVEMENT_CONFIG, config.input.clone())?.into_iter();

        let Achievements = archives
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
        Ok(Achievements)
    }

    fn deserialize(id: u32, mut buffer: Bytes) -> Self {
        let mut achievement = Self { id, ..Default::default() };

        loop {
            match buffer.get_u8() {
                0 => {
                    debug_assert_eq!(buffer.remaining(), 0, "{}", achievement);
                    break achievement;
                }
                1 => achievement.name = Some(buffer.get_padded_string()),
                2 => {
                    let is_ironman = buffer.get_u8();
                    assert_eq!(buffer.get_u8(), 0, "I'm unsure whether this is an invariant");

                    achievement.description = Some(buffer.get_padded_string());
                    if is_ironman == 2 {
                        assert_eq!(buffer.get_u8(), 1, "I'm unsure whether this is an invariant");
                        achievement.ironman_description = Some(buffer.get_padded_string());
                    }
                }
                3 => achievement.category = Some(buffer.get_u16()),
                4 => achievement.sprite_id = Some(buffer.get_smart32().unwrap()),
                5 => achievement.points = Some(buffer.get_u8()),
                6 => achievement.unknown_6 = Some(buffer.get_u16()),
                7 => achievement.reward = Some(buffer.get_padded_string()),
                8 => {
                    achievement.skill_requirements_1 = {
                        let count = buffer.get_unsigned_smart() as usize;
                        Some(
                            iter::repeat_with(|| MaybeIronmanSkillRequirement::deserialize(&mut buffer))
                                .take(count)
                                .collect(),
                        )
                    }
                }
                9 => {
                    achievement.unknown_9 = {
                        let count = buffer.get_u8() as usize;
                        Some(iter::repeat_with(|| VarbitRequirement10::deserialize(&mut buffer)).take(count).collect())
                    }
                }
                10 => {
                    achievement.unknown_10 = {
                        let count = buffer.get_u8() as usize;
                        Some(iter::repeat_with(|| VarbitRequirement10::deserialize(&mut buffer)).take(count).collect())
                    }
                }
                11 => {
                    achievement.previous_achievements = {
                        let count = buffer.get_u8() as usize;
                        Some(iter::repeat_with(|| buffer.get_uint(3) as u32).take(count).collect())
                    }
                }
                12 => {
                    achievement.skill_requirements_2 = {
                        let count = buffer.get_unsigned_smart() as usize;
                        Some(iter::repeat_with(|| SkillRequirement::deserialize(&mut buffer)).take(count).collect())
                    }
                }
                13 => {
                    achievement.unknown_13 = {
                        let count = buffer.get_unsigned_smart() as usize;
                        Some(
                            iter::repeat_with(|| MultipleVarbitsRequirement::deserialize(&mut buffer))
                                .take(count)
                                .collect(),
                        )
                    }
                }
                14 => {
                    achievement.subreqs_14 = {
                        let count = buffer.get_unsigned_smart() as usize;
                        Some(
                            iter::repeat_with(|| MultipleVarbitsRequirement::deserialize(&mut buffer))
                                .take(count)
                                .collect(),
                        )
                    }
                }
                15 => {
                    achievement.sub_achievements = {
                        let count = buffer.get_unsigned_smart() as usize;
                        Some(iter::repeat_with(|| buffer.get_uint(3) as u32).take(count).collect())
                    }
                }
                16 => achievement.sub_category = Some(buffer.get_u16()),
                17 => achievement.unknown_17 = Some(true),
                18 => achievement.hidden = Some(buffer.get_u8()),
                19 => achievement.free_to_play = Some(true),
                20 => {
                    achievement.quest_req_for_miniquests = {
                        let count = buffer.get_u8() as usize;
                        Some(iter::repeat_with(|| buffer.get_uint(3) as u32).take(count).collect())
                    }
                }
                21 => {
                    achievement.required_quest_ids = {
                        let count = buffer.get_u8() as usize;
                        Some(iter::repeat_with(|| buffer.get_uint(3) as u32).take(count).collect())
                    }
                }
                23 => {
                    achievement.reqs_23 = {
                        let count = buffer.get_unsigned_smart() as usize;
                        Some(
                            iter::repeat_with(|| PackedVarbitRequirement::deserialize(&mut buffer))
                                .take(count)
                                .collect(),
                        )
                    }
                }
                25 => {
                    achievement.reqs_25 = {
                        let count = buffer.get_unsigned_smart() as usize;
                        Some(
                            iter::repeat_with(|| PackedVarbitRequirement::deserialize(&mut buffer))
                                .take(count)
                                .collect(),
                        )
                    }
                }
                27 => achievement.unknown_27 = Some(true),
                28 => {
                    achievement.skill_req_count = {
                        let count = buffer.get_u8() as usize;
                        Some(iter::repeat_with(|| buffer.get_u8()).take(count).collect())
                    }
                }
                29 => achievement.unknown_29 = Some(buffer.get_u8()),
                30 => {
                    achievement.sub_req_count = {
                        let count = buffer.get_u8() as usize;
                        Some(iter::repeat_with(|| buffer.get_unsigned_smart()).take(count).collect())
                    }
                }
                31 => achievement.unknown_31 = Some(buffer.get_u8()),
                32 => achievement.unknown_32 = Some([buffer.get_u8(), buffer.get_u8(), buffer.get_u8()]),
                35 => achievement.unknown_35 = Some(true),
                37 => achievement.unknown_37 = Some(buffer.get_u8()),
                38 => achievement.unknown_38 = Some(buffer.get_u8()),
                missing => unimplemented!(
                    "Achievement::deserialize cannot deserialize opcode {} in id {}: \n {}",
                    missing,
                    id,
                    achievement
                ),
            }
        }
    }
}

pub mod achievement_fields_impl {
    #![allow(missing_docs)]
    use std::iter;

    use bytes::{Buf, Bytes};
    #[cfg(feature = "pyo3")]
    use pyo3::prelude::*;
    use serde::Serialize;

    use crate::{cache::buf::BufExtra, types::variables::Varbit};

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct VarbitRequirement {
        pub description: String,
        pub value: u16,
        pub varbit: Varbit,
        pub step_size: u8,
    }

    impl VarbitRequirement {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let _type = buffer.get_u8();
            let value = buffer.get_unsigned_smart();
            let description = buffer.get_padded_string();
            let step_size = buffer.get_u8();
            let varbit = Varbit::new(buffer.get_u16());
            Self {
                description,
                value,
                varbit,
                step_size,
            }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct VarbitRequirement10 {
        pub description: String,
        pub value: u32,
        pub varbit: Varbit,
        pub step_size: u8,
    }

    impl VarbitRequirement10 {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let _type = buffer.get_u8();
            let value = buffer.get_smart32().unwrap();
            let description = buffer.get_padded_string();
            let step_size = buffer.get_u8();
            let varbit = Varbit::new(buffer.get_u16());
            Self {
                description,
                value,
                varbit,
                step_size,
            }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct SkillRequirement {
        pub description: String,
        pub level: u8,
        pub skill: u16,
    }

    impl SkillRequirement {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let r#type = buffer.get_u8();
            assert_eq!(r#type, 0);
            let level = buffer.get_u8();
            let description = buffer.get_padded_string();
            let r#type2 = buffer.get_u8();
            assert_eq!(r#type2, 1);

            let skill = buffer.get_u16();
            Self { description, level, skill }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct MaybeIronmanSkillRequirement {
        pub is_ironman: bool,
        pub description: String,
        pub level: u8,
        pub skill: u16,
    }

    impl MaybeIronmanSkillRequirement {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let is_ironman = buffer.get_u8() == 1;

            let level = buffer.get_u8();
            let description = buffer.get_padded_string();

            let r#type = buffer.get_u8();
            assert_eq!(r#type, 1);

            let skill = buffer.get_u16();
            Self {
                is_ironman,
                description,
                level,
                skill,
            }
        }
    }
    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct MultipleVarbitsRequirement {
        value: u32,
        description: String,
        varbits: Vec<Varbit>,
    }

    impl MultipleVarbitsRequirement {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let _type = buffer.get_u8();
            let value = buffer.get_smart32().unwrap();
            let description = buffer.get_padded_string();
            let count = buffer.get_u8() as usize;
            let varbits = iter::repeat_with(|| Varbit::new(buffer.get_u16())).take(count).collect();
            Self { value, description, varbits }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct PartialVarbitRequirement {
        value: u32,
        step_size: u8,
        description: String,
        varbit: Varbit,
    }

    impl PartialVarbitRequirement {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let _type = buffer.get_u8();
            let varbit = Varbit::new(buffer.get_u16());
            let step_size = buffer.get_u8();
            let description = buffer.get_padded_string();
            let value = buffer.get_smart32().unwrap();

            Self {
                value,
                description,
                varbit,
                step_size,
            }
        }
    }

    #[cfg_eval]
    #[cfg_attr(feature = "pyo3", rs3cache_macros::pyo3_get_all)]
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Serialize, Debug, Clone)]
    pub struct PackedVarbitRequirement {
        value: u8,
        value2: u8,
        description: String,
        varbit: Varbit,
    }

    impl PackedVarbitRequirement {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let _type = buffer.get_u8();
            let varbit = Varbit::new(buffer.get_u16());
            let value = buffer.get_u8();
            let description = buffer.get_padded_string();
            let value2 = buffer.get_u8();

            Self {
                value,
                description,
                varbit,
                value2,
            }
        }
    }
}

use std::fmt::{self, Display, Formatter};

use achievement_fields_impl::*;

impl Display for Achievement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self).unwrap())
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Achievement {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Achievement({})", serde_json::to_string(self).unwrap()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Achievement({})", serde_json::to_string(self).unwrap()))
    }
}

/// Save the Achievement configs as `Achievement>.json`. Exposed as `--dump Achievement`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output)?;
    let mut achievement_configs = Achievement::dump_all(config)?.into_values().collect::<Vec<_>>();
    achievement_configs.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create(path!(config.output / "achievements.json"))?;
    let data = serde_json::to_string_pretty(&achievement_configs).unwrap();
    file.write_all(data.as_bytes())?;

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Config;

    #[test]
    fn t() -> CacheResult<()> {
        let config = Config::env();

        let _achievement_config = Achievement::dump_all(&config)?;

        Ok(())
    }
}
