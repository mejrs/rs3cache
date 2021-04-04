use crate::{
    cache::{
        buf::Buffer,
        index::CacheIndex,
        indextype::{ConfigType, IndexType},
    },
    structures::paramtable::ParamTable,
    types::variables::{Varbit, Varp, VarpOrVarbit},
    utils::error::CacheResult,
};
use itertools::izip;
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    iter,
};
/// Map element on the ingame world map.
///
/// This can be a text label, sprite, polygon or interactive.
#[derive(Debug, Default, Serialize)]
pub struct MapLabelConfig {
    /// File id of the [`MapLabelConfig`].
    pub id: u32,
    /// Text shown when the label is rightclicked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rightclick_1: Option<String>,
    /// Text shown when the label is rightclicked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rightclick_2: Option<String>,
    /// Contains a [`Varp`] or [`Varbit`] that controls whether the label is shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toggle_1: Option<Toggle>,
    /// Contains another [`Varp`] or [`Varbit`] that controls whether the label is shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toggle_2: Option<Toggle>,
    /// If present, the label is text on the map, with the given `String`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Text colour.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_colour_1: Option<(u8, u8, u8)>,
    /// Text colour 2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_colour_2: Option<(u8, u8, u8)>,
    /// Font size ( any of 0, 1, 2, 3), if the label is text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<u8>,
    /// The sprite shown on the map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprite: Option<u32>,
    /// The sprite shown on the map on mouseover.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_sprite: Option<u32>,
    /// The sprite shown on the map behind the main sprite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_sprite: Option<u32>,
    /// Unknown field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_7: Option<u8>,
    /// Unknown field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_8: Option<u8>,
    /// Customizes label creation in script 7590.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<u16>,
    /// Describes the polygon drawn on the map, if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polygon: Option<Polygon>,
    /// Unknown field.
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Unknown field.
    pub unknown_21: Option<Vec<u8>>,
    /// Unknown field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_22: Option<Vec<u8>>,
    /// Unknown field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_28: Option<u8>,
    /// Unknown field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_30: Option<u8>,
    /// Switch between the "new" and "legacy" icons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_switch: Option<LegacySwitch>,
    /// Contains additional options:
    /// # Possible param keys (non-exhaustive)
    /// | key | type | description |
    /// | :-------------: | :----------: | :-----------: |
    /// | 4147 | `u32` | Corresponds to a field of enum 2252|
    /// | 4148 | `u32` | Coordinate that the map pans to, if the label is clicked|
    /// | 4149 | `String` | Tooltip |
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub params: Option<ParamTable>,
}

impl MapLabelConfig {
    /// Returns a mapping of all [MapLabel] configurations.
    pub fn dump_all() -> CacheResult<HashMap<u32, MapLabelConfig>> {
        Ok(CacheIndex::new(IndexType::CONFIG)?
            .archive(ConfigType::MAPLABELS)?
            .take_files()
            .into_iter()
            .map(|(file_id, file)| (file_id, MapLabelConfig::deserialize(file_id, file)))
            .collect())
    }

    fn deserialize(id: u32, file: Vec<u8>) -> MapLabelConfig {
        let mut buffer = Buffer::new(file);
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

/// A polygon
#[derive(Debug, Default, Serialize)]
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
    fn deserialize(buffer: &mut Buffer) -> Polygon {
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

/// Controls whether the [`MapLabel`] is shown.
#[derive(Debug, Serialize)]
pub struct Toggle {
    /// The [`Varp`] or [`Varbit`] controlling the toggle.
    #[serde(flatten)]
    pub var: VarpOrVarbit,
    /// Lower bound of showing the [`MapLabel`].
    pub lower: u32,
    /// Upper bound of showing the [`MapLabel`].
    pub upper: u32,
}

impl Toggle {
    fn deserialize(buffer: &mut Buffer) -> Self {
        let varbit = Varbit::new(buffer.read_unsigned_short());
        let varp = Varp::new(buffer.read_unsigned_short());
        let var = VarpOrVarbit::new(varp, varbit);

        let lower = buffer.read_unsigned_int();
        let upper = buffer.read_unsigned_int();

        Self { var, lower, upper }
    }
}

/// Whether to show "new" or "legacy" map icon.
#[derive(Debug, Serialize)]
pub struct LegacySwitch {
    #[serde(flatten)]
    /// The [`Varp`] or [`Varbit`] controlling legacy toggle.
    pub var: VarpOrVarbit,
    /// Switch for the [`Varp`] or [`Varbit`].
    pub value: u8,
    /// A reference pointing to the default [`MapLabel`].
    pub default_reference: u16,
    /// A reference pointing to the legacy [`MapLabel`].
    pub legacy_reference: u16,
}

impl LegacySwitch {
    fn deserialize(buffer: &mut Buffer) -> Self {
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
#[derive(Debug, Serialize)]
pub struct PolygonPoint {
    /// Plane. Always zero.
    pub plane: u8,
    /// X-coordinate offset from the [`MapLabel`] location.
    pub dx: i16,
    /// Y-coordinate offset from the [`MapLabel`] location.
    pub dy: i16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dump_maplabels() -> CacheResult<()> {
        MapLabelConfig::dump_all()?;
        Ok(())
    }
}
