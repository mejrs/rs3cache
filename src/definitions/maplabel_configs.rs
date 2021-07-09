use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
    cache::{
        buf::  Buffer,
        index::CacheIndex,
        indextype::{ConfigType, IndexType},
    },
    structures::paramtable::ParamTable,
    utils::error::CacheResult,
};

/// Map element on the ingame world map.
///
/// This can be a text label, sprite, polygon or interactive.
#[cfg_attr(feature = "pyo3", pyclass)]
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize)]
pub struct MapLabelConfig {
    /// File id of the [`MapLabelConfig`].
    pub id: u32,
    /// Text shown when the label is rightclicked.
    pub rightclick_1: Option<String>,
    /// Text shown when the label is rightclicked.
    pub rightclick_2: Option<String>,
    /// A toggle that controls whether the label is shown.
    pub toggle_1: Option<Toggle>,
    /// Contains another toggle that controls whether the label is shown.
    pub toggle_2: Option<Toggle>,
    /// If present, the label is text on the map, with the given `String`.
    pub text: Option<String>,
    /// Text colour.
    pub label_colour_1: Option<(u8, u8, u8)>,
    /// Text colour 2.
    pub label_colour_2: Option<(u8, u8, u8)>,
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
    pub unknown_21: Option<Vec<u8>>,
    /// Unknown field.
    pub unknown_22: Option<Vec<u8>>,
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

#[cfg(feature = "pyo3")]
#[pymethods]
impl MapLabelConfig {
    #[getter]
    fn id(&self) -> PyResult<u32> {
        Ok(self.id)
    }

    #[getter]
    fn rightclick_1(&self) -> PyResult<Option<String>> {
        Ok(self.rightclick_1.clone())
    }

    #[getter]
    fn rightclick_2(&self) -> PyResult<Option<String>> {
        Ok(self.rightclick_2.clone())
    }
    #[getter]
    fn toggle_1(&self) -> PyResult<Option<Toggle>> {
        Ok(self.toggle_1)
    }

    #[getter]
    fn toggle_2(&self) -> PyResult<Option<Toggle>> {
        Ok(self.toggle_2)
    }

    #[getter]
    fn text(&self) -> PyResult<Option<String>> {
        Ok(self.text.clone())
    }

    #[getter]
    fn label_colour_1(&self) -> PyResult<Option<(u8, u8, u8)>> {
        Ok(self.label_colour_1)
    }

    #[getter]
    fn label_colour_2(&self) -> PyResult<Option<(u8, u8, u8)>> {
        Ok(self.label_colour_2)
    }

    #[getter]
    fn font_size(&self) -> PyResult<Option<u8>> {
        Ok(self.font_size)
    }

    #[getter]
    fn sprite(&self) -> PyResult<Option<u32>> {
        Ok(self.sprite)
    }

    #[getter]
    fn hover_sprite(&self) -> PyResult<Option<u32>> {
        Ok(self.hover_sprite)
    }

    #[getter]
    fn background_sprite(&self) -> PyResult<Option<u32>> {
        Ok(self.background_sprite)
    }

    #[getter]
    fn unknown_7(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_7)
    }
    #[getter]
    fn unknown_8(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_8)
    }

    #[getter]
    fn category(&self) -> PyResult<Option<u16>> {
        Ok(self.category)
    }

    #[getter]
    fn polygon(&self) -> PyResult<Option<Polygon>> {
        Ok(self.polygon.clone())
    }

    #[getter]
    fn unknown_21(&self) -> PyResult<Option<Vec<u8>>> {
        Ok(self.unknown_21.clone())
    }

    #[getter]
    fn unknown_22(&self) -> PyResult<Option<Vec<u8>>> {
        Ok(self.unknown_22.clone())
    }

    #[getter]
    fn unknown_28(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_28)
    }

    #[getter]
    fn unknown_30(&self) -> PyResult<Option<u8>> {
        Ok(self.unknown_30)
    }

    #[getter]
    fn legacy_switch(&self) -> PyResult<Option<LegacySwitch>> {
        Ok(self.legacy_switch)
    }

    #[getter]
    fn params(&self) -> PyResult<Option<ParamTable>> {
        Ok(self.params.clone())
    }
}

impl MapLabelConfig {
    /// Returns a mapping of all [`MapLabelConfig`]s.
    pub fn dump_all() -> CacheResult<HashMap<u32, MapLabelConfig>> {
        Ok(CacheIndex::new(IndexType::CONFIG)?
            .archive(ConfigType::MAPLABELS)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, MapLabelConfig::deserialize(file_id, file)))
            .collect())
    }

    fn deserialize(id: u32, file: Vec<u8>) -> MapLabelConfig {
        let mut buffer =  Buffer::new(file);
        let mut maplabel = MapLabelConfig { id, ..Default::default() };

        loop {
            match buffer.read_unsigned_byte() {
                0 => {
                    debug_assert_eq!(buffer.remaining(), 0);
                    break maplabel;
                }
                1 => maplabel.sprite = Some(buffer.read_smart32().unwrap()),
                2 => maplabel.hover_sprite = Some(buffer.read_smart32().unwrap()),
                3 => maplabel.text = Some(buffer.read_string()),
                4 => maplabel.label_colour_1 = Some(buffer.read_rgb()),
                5 => maplabel.label_colour_2 = Some(buffer.read_rgb()),
                6 => maplabel.font_size = Some(buffer.read_unsigned_byte()),
                7 => maplabel.unknown_7 = Some(buffer.read_unsigned_byte()),
                8 => maplabel.unknown_8 = Some(buffer.read_unsigned_byte()),
                9 => maplabel.toggle_1 = Some(Toggle::deserialize(&mut buffer)),
                10 => maplabel.rightclick_1 = Some(buffer.read_string()),
                15 => maplabel.polygon = Some(Polygon::deserialize(&mut buffer)),
                17 => maplabel.rightclick_2 = Some(buffer.read_string()),
                19 => maplabel.category = Some(buffer.read_unsigned_short()),
                20 => maplabel.toggle_2 = Some(Toggle::deserialize(&mut buffer)),
                21 => maplabel.unknown_21 = Some(buffer.read_n_bytes(4)),
                22 => maplabel.unknown_22 = Some(buffer.read_n_bytes(4)),
                25 => maplabel.background_sprite = Some(buffer.read_smart32().unwrap()),
                26 => maplabel.legacy_switch = Some(LegacySwitch::deserialize(&mut buffer)),
                28 => maplabel.unknown_28 = Some(buffer.read_unsigned_byte()),
                30 => maplabel.unknown_30 = Some(buffer.read_unsigned_byte()),
                249 => maplabel.params = Some(ParamTable::deserialize(&mut buffer)),
                other => unimplemented!("{}", other),
            }
        }
    }
}

///Save the maplabels as `maplabels.json`. Exposed as `--dump maplabels`.
pub fn export() -> CacheResult<()> {
    fs::create_dir_all("out")?;
    let mut labels = MapLabelConfig::dump_all()?.into_values().collect::<Vec<_>>();
    labels.sort_unstable_by_key(|loc| loc.id);

    let mut file = File::create("out/maplabels.json")?;
    let data = serde_json::to_string_pretty(&labels)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

/// Defines the structs used as fields of [`MapLabelConfig`],
pub mod maplabel_config_fields {
    #![allow(missing_docs)]
    use std::iter;

    use itertools::izip;
    #[cfg(feature = "pyo3")]
    use pyo3::prelude::*;
    use serde::Serialize;

    use crate::{
        cache::buf::  Buffer,
        types::variables::{Varbit, Varp, VarpOrVarbit},
    };
    /// A polygon
    #[cfg_attr(feature = "pyo3", pyclass)]
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
        pub fn deserialize(buffer: &mut  Buffer<Vec<u8>>) -> Polygon {
            let point_count = buffer.read_unsigned_byte() as usize;
            let xy = iter::repeat_with(|| (buffer.read_short(), buffer.read_short()))
                .take(point_count)
                .collect::<Vec<(i16, i16)>>();

            let colour = [
                buffer.read_unsigned_byte(),
                buffer.read_unsigned_byte(),
                buffer.read_unsigned_byte(),
                buffer.read_unsigned_byte(),
            ];
            assert_eq!(buffer.read_unsigned_byte(), 1_u8);

            let background_colour = [
                buffer.read_unsigned_byte(),
                buffer.read_unsigned_byte(),
                buffer.read_unsigned_byte(),
                buffer.read_unsigned_byte(),
            ];

            let planes = iter::repeat_with(|| buffer.read_unsigned_byte()).take(point_count).collect::<Vec<_>>();
            let points = izip!(planes, xy).map(|(plane, (dx, dy))| PolygonPoint { plane, dx, dy }).collect();

            Polygon {
                colour,
                background_colour,
                points,
            }
        }
    }

    /// Controls whether the [`MapLabelConfig`](super::MapLabelConfig) is shown.
    #[cfg_attr(feature = "pyo3", pyclass)]
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
        pub fn deserialize(buffer: &mut  Buffer<Vec<u8>>) -> Self {
            let varbit = Varbit::new(buffer.read_unsigned_short());
            let varp = Varp::new(buffer.read_unsigned_short());
            let var = VarpOrVarbit::new(varp, varbit);

            let lower = buffer.read_unsigned_int();
            let upper = buffer.read_unsigned_int();

            Self { var, lower, upper }
        }
    }

    /// Whether to show "new" or "legacy" map icon.
    #[cfg_attr(feature = "pyo3", pyclass)]
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
        pub fn deserialize(buffer: &mut  Buffer<Vec<u8>>) -> Self {
            let varbit = Varbit::new(buffer.read_unsigned_short());
            let varp = Varp::new(buffer.read_unsigned_short());
            let var = VarpOrVarbit::new(varp, varbit);

            // always 0 or 1 (boolean)
            let value = buffer.read_unsigned_byte();
            let default_reference = buffer.read_unsigned_short();
            let legacy_reference = buffer.read_unsigned_short();

            Self {
                var,
                value,
                default_reference,
                legacy_reference,
            }
        }
    }

    /// Points that span a [`Polygon`].
    #[cfg_attr(feature = "pyo3", pyclass)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dump_maplabels() -> CacheResult<()> {
        MapLabelConfig::dump_all()?;
        Ok(())
    }
}
