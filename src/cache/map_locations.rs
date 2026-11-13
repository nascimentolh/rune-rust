//! A map region's locations (objects: walls, trees, tables...): file 0 of archive `l{x}_{y}` in
//! index 5, encrypted with the region's XTEA key (`MapRegion`, `Index.getFile(archive, 0, key)`).
//! Port of the file's layout as `Class329_Sub1.method12461` reads it:
//!
//! - the object id as running sums (`readSum`, starting from −1), ending at 0;
//! - for each id, its positions as running sums of smarts − 1 (`readSmart`), ending at 0:
//!   plane << 12 | x << 6 | y inside the region;
//! - after each position a byte: type << 2 | rotation.

use super::buffer::Reader;
use super::error::Result;

#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    /// The object definition (index 16).
    pub id: u32,
    /// Tile inside the region (0..64) and plane.
    pub x: u8,
    pub y: u8,
    pub plane: u8,
    /// `ObjectType` id (0..=22: walls, wall decorations, scenery, ground decoration...).
    pub kind: u8,
    /// Quarter turns.
    pub rotation: u8,
}

/// Every location in a region's file.
pub fn decode(data: &[u8]) -> Result<Vec<Location>> {
    let mut r = Reader::new(data);
    let mut out = Vec::new();
    let mut id: i64 = -1;
    loop {
        let step = r.smart_sum()?;
        if step == 0 {
            return Ok(out);
        }
        id += step as i64;
        let mut position = 0u32;
        loop {
            let step = r.smart()?;
            if step == 0 {
                break;
            }
            position += step - 1;
            let info = r.u8()?;
            out.push(Location {
                id: id as u32,
                x: (position >> 6 & 0x3f) as u8,
                y: (position & 0x3f) as u8,
                plane: (position >> 12) as u8,
                kind: info >> 2,
                rotation: info & 3,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locations_read_their_running_sums() {
        // Object 1 (−1 + 2) at plane 1, x 3, y 4 (position 4096 + 196 → smart 4293 + 1 as two
        // bytes), type 10 rotation 2; then object 3 (+2) at (0, 0, 5) (smart 6), type 22.
        let pos = 4096 + (3 << 6) + 4 + 1;
        let mut data = vec![2, (0x80 | (pos >> 8)) as u8, pos as u8, (10 << 2) | 2, 0];
        data.extend([2, 6, 22 << 2, 0, 0]);
        let l = decode(&data).unwrap();
        assert_eq!(l[0], Location { id: 1, x: 3, y: 4, plane: 1, kind: 10, rotation: 2 });
        assert_eq!(l[1], Location { id: 3, x: 0, y: 5, plane: 0, kind: 22, rotation: 0 });
    }
}
