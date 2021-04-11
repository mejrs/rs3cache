//! The names of various indexes and archives.

/// Enumeration of all index types.
pub struct IndexType;

#[allow(dead_code)]
impl IndexType {
    /// Unimplemented.
    pub const BASES: u32 = 1;
    /// Contains various smaller [`ConfigType`] definitions.
    pub const CONFIG: u32 = 2;
    /// Unimplemented.
    pub const INTERFACES: u32 = 3;
    /// Contains [`MapSquare`](crate::definitions::mapsquares::MapSquare) definitions.
    pub const MAPSV2: u32 = 5;
    /// Discontinued.
    const MODELS: u32 = 7;
    /// Contains [`Sprite`](crate::definitions::sprites::Sprite) definitions.
    pub const SPRITES: u32 = 8;
    /// Unimplemented.
    pub const BINARY: u32 = 10;
    /// Contains client side scripts in a bytecode-like format (cs2). Unimplemented.
    pub const SCRIPTS: u32 = 12;
    /// Unimplemented.
    pub const FONTMETRICS: u32 = 13;
    /// Unimplemented.
    pub const VORBIS: u32 = 14;
    /// Contains the [`LocationConfig`](crate::definitions::location_configs::LocationConfig) definitions.
    pub const LOC_CONFIG: u32 = 16;
    /// Unimplemented.
    pub const ENUM_CONFIG: u32 = 17;
    /// Contains the [`NpcConfig`](crate::definitions::npc_configs::NpcConfig) definitions.
    pub const NPC_CONFIG: u32 = 18;
    /// Unimplemented.
    pub const OBJ_CONFIG: u32 = 19;
    /// Unimplemented.
    pub const SEQ_CONFIG: u32 = 20;
    /// Unimplemented.
    pub const SPOT_CONFIG: u32 = 21;
    /// Unimplemented.
    pub const STRUCT_CONFIG: u32 = 22;
    /// Contains [`MapZone`](crate::definitions::worldmaps::MapZone),
    /// [`MapPastes`](crate::definitions::worldmaps::MapPastes) definitions,
    /// as well as PNG images of the world map.
    pub const WORLDMAP: u32 = 23;
    /// Unimplemented.
    pub const QUICKCHAT: u32 = 24;
    /// Unimplemented.
    pub const GLOBAL_QUICKCHAT: u32 = 25;
    /// Unimplemented.
    pub const MATERIALS: u32 = 26;
    /// Unimplemented.
    pub const PARTICLES: u32 = 27;
    /// Unimplemented.
    pub const DEFAULTS: u32 = 28;
    /// Unimplemented.
    pub const BILLBOARDS: u32 = 29;
    /// Discontinued.
    const DLLS: u32 = 30;
    /// Discontinued.
    const SHADERS: u32 = 31;
    /// Discontinued.
    const LOADINGSPRITES: u32 = 32;
    /// Discontinued.
    const LOADINGSCREEN: u32 = 33;
    /// Discontinued.
    const LOADINGSPRITESRAW: u32 = 34;
    /// Discontinued.
    const CUTSCENES: u32 = 35;
    /// Unimplemented.
    pub const AUDIOSTREAMS: u32 = 40;
    /// Unimplemented.
    pub const WORLDMAPAREAS: u32 = 41;
    /// Unimplemented.
    pub const WORLDMAPLABELS: u32 = 42;
    /// Unimplemented.
    pub const MODELSRT7: u32 = 47;
    /// Unimplemented.
    pub const ANIMSRT7: u32 = 48;
    /// Unimplemented.
    pub const DBTABLEINDEX: u32 = 49;
    /// Unimplemented.
    pub const TEXTURES: u32 = 52;
    /// Unimplemented.
    pub const TEXTURES_PNG: u32 = 53;
    /// Unimplemented.
    pub const TEXTURES_PNG_MIPPED: u32 = 54;
    /// Unimplemented.
    pub const TEXTURES_ETC: u32 = 55;
    /// Unimplemented.
    pub const ANIMS_KEYFRAMES: u32 = 56;
    /// Unimplemented.
    pub const ACHIEVEMENT_CONFIG: u32 = 57;
}

impl IndexType {
    /// Iterate over all indextypes.
    pub fn iterator() -> std::vec::IntoIter<u32> {
        vec![
            1, 2, 3, 5, 8, 10, 12, 13, 14, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 40, 41, 42, 47, 48, 49, 52, 54, 55, 56, 57,
        ]
        .into_iter()
    }
}

/// Enumeration of all archives in the Config (2) index.
pub struct ConfigType;

impl ConfigType {
    /// Contains [`Underlay`](crate::definitions::underlays::Underlay) definitions.
    pub const UNDERLAYS: u32 = 1;
    /// Unimplemented.
    pub const HUNT: u32 = 2;
    /// Unimplemented.
    pub const IDENTITY_KIT: u32 = 3;
    /// Contains [`Overlay`](crate::definitions::overlays::Overlay) definitions.
    pub const OVERLAYS: u32 = 4;
    /// Unimplemented.
    pub const INVENTORY: u32 = 5;
    /// Unimplemented.
    pub const UNKNOWN_7: u32 = 7;
    /// Unimplemented.
    pub const TOOLTIPS: u32 = 11;
    /// Unimplemented.
    pub const AREA: u32 = 18;
    /// Unimplemented.
    pub const SKYBOX: u32 = 29;
    /// Unimplemented.
    pub const LIGHT: u32 = 31;
    /// Unimplemented.
    pub const BASE_ANIMATION_SET: u32 = 32;
    /// Unimplemented.
    pub const CURSORS: u32 = 33;
    /// Contains [`MapScene`](crate::definitions::mapscenes::MapScene).
    pub const MAPSCENES: u32 = 34;
    /// Unimplemented.
    pub const QUESTS: u32 = 35;
    /// Unimplemented.
    pub const MAPLABELS: u32 = 36;
    /// Unimplemented.
    pub const DBTABLE: u32 = 40;
    /// Unimplemented.
    pub const DBROWS: u32 = 41;
    /// Unimplemented.
    pub const UNKNOWN_42: u32 = 42;
    /// Unimplemented.
    pub const HITSPLATS: u32 = 46;
    /// Unimplemented.
    pub const UNKNOWN_48: u32 = 48;
    /// Unimplemented.
    pub const UNKNOWN_49: u32 = 59;
    /// Unimplemented.
    pub const PLAYER: u32 = 60;
    /// Unimplemented.
    pub const NPC: u32 = 61;
    /// Unimplemented.
    pub const CLIENT: u32 = 62;
    /// Unimplemented.
    pub const WORLD: u32 = 63;
    /// Unimplemented.
    pub const REGION: u32 = 64;
    /// Unimplemented.
    pub const OBJECT: u32 = 65;
    /// Unimplemented.
    pub const CLAN: u32 = 66;
    /// Unimplemented.
    pub const CLAN_SETTING: u32 = 67;
    /// Unimplemented.
    pub const CAMPAIGN: u32 = 68;
    /// Unimplemented.
    pub const VARBITS: u32 = 69;
    /// Unimplemented.
    pub const UNKNOWN_70: u32 = 70;
    /// Unimplemented.
    pub const HEADBAR: u32 = 72;
    /// Unimplemented.
    pub const UNKNOWN_73: u32 = 73;
    /// Unimplemented.
    pub const UNKNOWN_75: u32 = 75;
    /// Unimplemented.
    pub const UNKNOWN_76: u32 = 76;
    /// Unimplemented.
    pub const UNKNOWN_77: u32 = 77;
    /// Unimplemented.
    pub const SEQGROUP: u32 = 80;
    /// Unimplemented.
    pub const UNKNOWN_83: u32 = 83;
}
