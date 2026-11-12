//! The first game packet of a world login: `REGION` (server opcode 85, size −2 = a u16 length),
//! which the client reads right after the login reply (`Login.java`, login stages 167 → 173 →
//! 156) before it shows the world.
//!
//! Its payload, as the client reads it:
//! - `Class197.method3203` (player positions, in bits): 30 bits for this player
//!   (plane << 28 | x << 14 | y), then 18 bits for each other index 1..2047
//!   (plane << 16 | region x << 8 | region y; 0 = nobody known yet);
//! - `MapRegion.loadMapSceneNormal`: map size (0 = 104 tiles), the centre chunk x and y (u16),
//!   a "force reload" byte, then four XTEA key ints for every map region around the centre:
//!   region x from (chunk x − 6) / 8 to (chunk x + 6) / 8, and within it region y the same way.
//!
//! Where a new player starts is this server's choice (`SPAWN`).

use crate::cache::xtea::Key;
use std::collections::HashMap;

/// `ServerProt.REGION`.
pub const REGION: u32 = 85;
/// Where players start: (x, y, plane). Estimated — a common starting spot, not from the client.
pub const SPAWN: (i32, i32, i32) = (3222, 3218, 0);
/// `MapSize.SIZE_104.index`, and its tiles >> 4 (`sizeX >> 4`).
const MAP_SIZE: u8 = 0;
const HALF_SPAN: i32 = 104 >> 4;

/// MSB-first bit writer (`ByteBuf.Bit.writeBits`).
#[derive(Default)]
struct Bits {
    bytes: Vec<u8>,
    at: usize,
}

impl Bits {
    fn write(&mut self, count: u32, value: u32) {
        for i in (0..count).rev() {
            if self.at.is_multiple_of(8) {
                self.bytes.push(0);
            }
            if value >> i & 1 == 1 {
                let last = self.bytes.len() - 1;
                self.bytes[last] |= 0x80 >> (self.at % 8);
            }
            self.at += 1;
        }
    }
}

/// The map regions `loadMapSceneNormal` asks keys for, in its order.
pub fn regions(chunk_x: i32, chunk_y: i32) -> Vec<u32> {
    let mut out = Vec::new();
    for rx in (chunk_x - HALF_SPAN) / 8..=(chunk_x + HALF_SPAN) / 8 {
        for ry in (chunk_y - HALF_SPAN) / 8..=(chunk_y + HALF_SPAN) / 8 {
            out.push((rx << 8 | ry) as u32);
        }
    }
    out
}

/// The `REGION` payload for player `index` at `at`, with the map keys by region id.
pub fn region_payload(index: u16, at: (i32, i32, i32), keys: &HashMap<u32, Key>) -> Vec<u8> {
    let (x, y, plane) = at;
    let mut bits = Bits::default();
    bits.write(30, (plane << 28 | x << 14 | y) as u32);
    for other in 1..2048u16 {
        if other != index {
            bits.write(18, 0);
        }
    }
    let mut out = bits.bytes;
    let (cx, cy) = (x >> 3, y >> 3);
    out.push(MAP_SIZE);
    out.extend((cx as u16).to_be_bytes());
    out.extend((cy as u16).to_be_bytes());
    out.push(1);
    for region in regions(cx, cy) {
        for k in keys.get(&region).copied().unwrap_or_default() {
            out.extend(k.to_be_bytes());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_are_written_high_bit_first() {
        let mut b = Bits::default();
        b.write(3, 0b101);
        b.write(6, 0b111111);
        assert_eq!(b.bytes, vec![0b1011_1111, 0b1000_0000]);
    }

    #[test]
    fn the_payload_holds_the_spawn_and_one_key_per_region() {
        let p = region_payload(1, SPAWN, &HashMap::new());
        let bits_len = (30 + 2046 * 18_usize).div_ceil(8);
        let first = u32::from_be_bytes([p[0], p[1], p[2], p[3]]) >> 2;
        assert_eq!(first, (SPAWN.0 << 14 | SPAWN.1) as u32);
        let n = regions(SPAWN.0 >> 3, SPAWN.1 >> 3).len();
        assert_eq!(p.len(), bits_len + 6 + n * 16);
    }
}
