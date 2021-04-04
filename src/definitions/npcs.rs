use crate::cache::buf::Buffer;
use serde::Serialize;

/// A non player character.
#[derive(Copy, Clone, Debug, Serialize)]
pub struct Npc {
    /// The plane a.k.a elevation.
    pub plane: u8,

    /// The horizontal [`MapSquare`](crate::definitions::mapsquares::MapSquare) coordinate.
    pub i: u8,

    /// The vertical [`MapSquare`](crate::definitions::mapsquares::MapSquare) coordinate.
    pub j: u8,

    /// The horizontal coordinate inside its [`MapSquare`](crate::definitions::mapsquares::MapSquare).
    pub x: u8,

    /// The vertical coordinate inside its [`MapSquare`](crate::definitions::mapsquares::MapSquare).
    pub y: u8,

    /// The id corresponding to its [`NpcConfig`](crate::definitions::npc_configs::NpcConfig).
    pub id: u32,
}

impl Npc {
    /// Constructor for [`Npc`].
    pub fn deserialize(i: u8, j: u8, file: Vec<u8>) -> Vec<Npc> {
        let length = file.len();

        let mut buffer = Buffer::new(file);
        let mut npcs = Vec::with_capacity(length / 4);

        for _ in 0..(length / 4) {
            let value = buffer.read_unsigned_short();

            let plane = (value >> 14) as u8;
            let x = (value >> 7 & 0x3F) as u8;
            let y = (value & 0x3F) as u8;

            let id = buffer.read_unsigned_short() as u32;

            let npc = Npc { plane, i, j, x, y, id };

            npcs.push(npc);
        }

        debug_assert_eq!(buffer.remaining(), 0);

        npcs
    }
}
