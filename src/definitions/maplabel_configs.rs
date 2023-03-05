use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
};

use ::error::Context;
use bytes::{Buf, Bytes};
use path_macro::path;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rs3cache_backend::{
    buf::{BufExtra, JString},
    error::{self, CacheResult},
    index::CacheIndex,
};
use serde::Serialize;

use crate::{
    definitions::indextype::{ConfigType, IndexType},
    structures::paramtable::ParamTable,
};

/// Map element on the ingame world map.
///
/// This can be a text label, sprite, polygon or interactive.

#[cfg_attr(feature = "pyo3", pyclass(frozen, get_all))]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct MapLabelConfig {
    /// File id of the [`MapLabelConfig`].
    pub id: u32,
    /// Text shown when the label is rightclicked.
    pub rightclick_1: Option<JString<Bytes>>,
    /// Text shown when the label is rightclicked.
    pub rightclick_2: Option<JString<Bytes>>,
    /// A toggle that controls whether the label is shown.
    pub toggle_1: Option<Toggle>,
    /// Contains another toggle that controls whether the label is shown.
    pub toggle_2: Option<Toggle>,
    /// If present, the label is text on the map, with the given `String`.
    pub text: Option<JString<Bytes>>,
    /// Text colour.
    pub label_colour_1: Option<[u8; 3]>,
    /// Text colour 2.
    pub label_colour_2: Option<[u8; 3]>,
    /// Font size ( any of 0, 1, 2, 3), if the label is text.
    pub font_size: Option<u8>,
    /// The sprite shown on the map.
    pub sprite: Option<u32>,
    /// The sprite shown on the map on mouseover.
    pub hover_sprite: Option<u32>,
    /// The sprite shown on the map behind the main sprite.
    pub background_sprite: Option<u32>,
    /// Unknown field.
    pub unknown_7: Option<u8>,
    /// Unknown field.
    pub unknown_8: Option<u8>,
    /// Customizes label creation in script 7590.
    pub category: Option<u16>,
    /// Describes the polygon drawn on the map, if present.
    pub polygon: Option<Polygon>,
    /// Unknown field.
    pub unknown_21: Option<[u8; 4]>,
    /// Unknown field.
    pub unknown_22: Option<[u8; 4]>,
    /// Unknown field.
    pub unknown_28: Option<u8>,
    /// Unknown field.
    pub unknown_30: Option<u8>,
    /// Switch between the "new" and "legacy" icons.
    pub legacy_switch: Option<LegacySwitch>,
    /// Contains additional options:
    /// # Possible param keys (non-exhaustive)
    /// | key | type | description |
    /// | :-------------: | :----------: | :-----------: |
    /// | 4147 | `u32` | Corresponds to a field of enum 2252|
    /// | 4148 | `u32` | Coordinate that the map pans to, if the label is clicked|
    /// | 4149 | `String` | Tooltip |
    #[serde(flatten)]
    pub params: Option<ParamTable>,
}

impl MapLabelConfig {
    /// Returns a mapping of all [`MapLabelConfig`]s.
    pub fn dump_all(config: &crate::cli::Config) -> CacheResult<BTreeMap<u32, MapLabelConfig>> {
        Ok(CacheIndex::new(IndexType::CONFIG, config.input.clone())?
            .archive(ConfigType::MAPLABELS)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, MapLabelConfig::deserialize(file_id, file)))
            .collect())
    }

    fn deserialize(id: u32, mut buffer: Bytes) -> MapLabelConfig {
        let mut maplabel = MapLabelConfig { id, ..Default::default() };

        loop {
            match buffer.get_u8() {
                0 => {
                    debug_assert!(!buffer.has_remaining(), "{buffer:?}");
                    break maplabel;
                }
                1 => maplabel.sprite = Some(buffer.get_smart32().unwrap()),
                2 => maplabel.hover_sprite = Some(buffer.get_smart32().unwrap()),
                3 => maplabel.text = Some(buffer.get_string()),
                4 => maplabel.label_colour_1 = Some(buffer.get_rgb()),
                5 => maplabel.label_colour_2 = Some(buffer.get_rgb()),
                6 => maplabel.font_size = Some(buffer.get_u8()),
                7 => maplabel.unknown_7 = Some(buffer.get_u8()),
                8 => maplabel.unknown_8 = Some(buffer.get_u8()),
                9 => maplabel.toggle_1 = Some(Toggle::deserialize(&mut buffer)),
                10 => maplabel.rightclick_1 = Some(buffer.get_string()),
                15 => maplabel.polygon = Some(Polygon::deserialize(&mut buffer)),
                17 => maplabel.rightclick_2 = Some(buffer.get_string()),
                19 => maplabel.category = Some(buffer.get_u16()),
                20 => maplabel.toggle_2 = Some(Toggle::deserialize(&mut buffer)),
                21 => maplabel.unknown_21 = Some(buffer.get_array()),
                22 => maplabel.unknown_22 = Some(buffer.get_array()),
                25 => maplabel.background_sprite = Some(buffer.get_smart32().unwrap()),
                26 => maplabel.legacy_switch = Some(LegacySwitch::deserialize(&mut buffer)),
                28 => maplabel.unknown_28 = Some(buffer.get_u8()),
                30 => maplabel.unknown_30 = Some(buffer.get_u8()),
                249 => maplabel.params = Some(ParamTable::deserialize(&mut buffer)),
                other => unimplemented!("{}", other),
            }
        }
    }
}

///Save the maplabels as `maplabels.json`. Exposed as `--dump maplabels`.
pub fn export(config: &crate::cli::Config) -> CacheResult<()> {
    fs::create_dir_all(&config.output).with_context(|| error::Io { path: config.output.clone() })?;
    let mut labels = MapLabelConfig::dump_all(config)?.into_values().collect::<Vec<_>>();
    labels.sort_unstable_by_key(|loc| loc.id);
    let path = path!(config.output / "map_labels.json");

    let mut file = File::create(&path).with_context(|| error::Io { path: path.clone() })?;
    let data = serde_json::to_string_pretty(&labels).unwrap();
    file.write_all(data.as_bytes()).context(error::Io { path })?;
    Ok(())
}

/// Defines the structs used as fields of [`MapLabelConfig`],
pub mod maplabel_config_fields {

    use std::iter;

    use bytes::{Buf, Bytes};
    use itertools::izip;
    #[cfg(feature = "pyo3")]
    use pyo3::prelude::*;
    use serde::Serialize;

    use crate::types::variables::{Varbit, Varp, VarpOrVarbit};
    /// A polygon
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Debug, Clone, Default, Serialize)]
    pub struct Polygon {
        /// Colour of the polygon.
        pub colour: [u8; 4],

        /// Fill colour of the polygon.
        /// # Notes
        /// A value of `[255, 0, 255, 255]` indicates transparency.
        pub background_colour: [u8; 4],

        /// The coordinates spanning the `Polygon`.
        pub points: Vec<PolygonPoint>,
    }

    impl Polygon {
        pub fn deserialize(buffer: &mut Bytes) -> Polygon {
            let point_count = buffer.get_u8() as usize;
            let xy = iter::repeat_with(|| (buffer.get_i16(), buffer.get_i16()))
                .take(point_count)
                .collect::<Vec<(i16, i16)>>();

            let colour = [buffer.get_u8(), buffer.get_u8(), buffer.get_u8(), buffer.get_u8()];
            assert_eq!(buffer.get_u8(), 1_u8);

            let background_colour = [buffer.get_u8(), buffer.get_u8(), buffer.get_u8(), buffer.get_u8()];

            let planes = iter::repeat_with(|| buffer.get_u8()).take(point_count).collect::<Vec<_>>();
            let points = izip!(planes, xy).map(|(plane, (dx, dy))| PolygonPoint { plane, dx, dy }).collect();

            Polygon {
                colour,
                background_colour,
                points,
            }
        }
    }

    /// Controls whether the [`MapLabelConfig`](super::MapLabelConfig) is shown.
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Debug, Clone, Copy, Serialize)]
    pub struct Toggle {
        /// The [`Varp`] or [`Varbit`] controlling the toggle.

        #[serde(flatten)]
        pub var: VarpOrVarbit,

        /// Lower bound of showing the [`MapLabelConfig`](super::MapLabelConfig).
        pub lower: u32,

        /// Upper bound of showing the [`MapLabelConfig`](super::MapLabelConfig).
        pub upper: u32,
    }

    impl Toggle {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let varbit = Varbit::new(buffer.get_u16());
            let varp = Varp::new(buffer.get_u16());
            let var = VarpOrVarbit::new(varp, varbit);

            let lower = buffer.get_u32();
            let upper = buffer.get_u32();

            Self { var, lower, upper }
        }
    }

    /// Whether to show "new" or "legacy" map icon.
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Debug, Clone, Copy, Serialize)]
    pub struct LegacySwitch {
        /// The [`Varp`] or [`Varbit`] controlling legacy toggle.

        #[serde(flatten)]
        pub var: VarpOrVarbit,

        /// Switch for the [`Varp`] or [`Varbit`].
        pub value: u8,

        /// A reference pointing to the default [`MapLabelConfig`](super::MapLabelConfig).
        pub default_reference: u16,

        /// A reference pointing to the legacy [`MapLabelConfig`](super::MapLabelConfig).
        pub legacy_reference: u16,
    }

    impl LegacySwitch {
        pub fn deserialize(buffer: &mut Bytes) -> Self {
            let varbit = Varbit::new(buffer.get_u16());
            let varp = Varp::new(buffer.get_u16());
            let var = VarpOrVarbit::new(varp, varbit);

            // always 0 or 1 (boolean)
            let value = buffer.get_u8();
            let default_reference = buffer.get_u16();
            let legacy_reference = buffer.get_u16();

            Self {
                var,
                value,
                default_reference,
                legacy_reference,
            }
        }
    }

    /// Points that span a [`Polygon`].
    #[cfg_attr(feature = "pyo3", pyclass(frozen))]
    #[derive(Debug, Clone, Copy, Serialize)]
    pub struct PolygonPoint {
        /// Plane. Always zero.
        pub plane: u8,

        /// X-coordinate offset from the [`MapLabelConfig`](super::MapLabelConfig) position.
        pub dx: i16,

        /// Y-coordinate offset from the [`MapLabelConfig`](super::MapLabelConfig) position.
        pub dy: i16,
    }
}

use maplabel_config_fields::*;

#[cfg(all(test, any(feature = "rs3", feature = "osrs")))]
mod tests {
    use super::*;

    #[test]
    fn dump_maplabels() -> CacheResult<()> {
        let config = crate::cli::Config::env();

        MapLabelConfig::dump_all(&config)?;
        Ok(())
    }
}
