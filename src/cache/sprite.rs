//! Jagex sprite archives: one or more palette-indexed images in one file.
//! Port of `SpriteDefinitions.decode`.
//!
//! The table is at the end of the file:
//!   ... · palette (palette size - 1 colours, u24 each) · max width u16 · max height u16 ·
//!   palette size - 1 (u8) · min x[count] · min y[count] · width[count] · height[count] · count u16
//! Pixel data starts at offset 0: per sprite a flags byte (1 = stored column by column,
//! 2 = an alpha plane follows), then the palette indexes (and alpha bytes).
//! Palette index 0 is transparent.

use super::buffer::Reader;
use super::error::{corrupt, Result};

#[derive(Clone, Debug)]
pub struct Sprite {
    /// Offset of the image inside its full (max width × max height) frame.
    pub min_x: i32,
    pub min_y: i32,
    /// The full frame (`getMaxWidth` / `getMaxHeight`): the image plus its padding.
    pub frame_width: i32,
    pub frame_height: i32,
    pub width: usize,
    pub height: usize,
    /// ARGB per pixel; alpha 0 where the palette index is 0.
    pub pixels: Vec<u32>,
}

pub fn decode(data: &[u8]) -> Result<Vec<Sprite>> {
    if data.len() < 2 {
        return corrupt("sprite archive too short");
    }
    let mut r = Reader::new(data);
    r.seek(data.len() - 2);
    let count = r.u16()? as usize;
    let Some(table) = data.len().checked_sub(7 + count * 8) else { return corrupt("sprite table doesn't fit") };
    r.seek(table);
    let frame_width = r.u16()? as i32;
    let frame_height = r.u16()? as i32;
    let palette_len = r.u8()? as usize + 1;
    let mut columns = [Vec::with_capacity(count), Vec::with_capacity(count), Vec::with_capacity(count), Vec::with_capacity(count)];
    for column in &mut columns {
        for _ in 0..count {
            column.push(r.u16()? as usize);
        }
    }
    let [min_x, min_y, widths, heights] = columns;

    let Some(palette_at) = table.checked_sub((palette_len - 1) * 3) else { return corrupt("sprite palette doesn't fit") };
    r.seek(palette_at);
    let mut palette = vec![0u32; palette_len];
    for colour in palette.iter_mut().skip(1) {
        // A stored 0 would be transparent, so the client turns it into 1.
        *colour = r.u24()?.max(1);
    }

    r.seek(0);
    let mut sprites = Vec::with_capacity(count);
    for i in 0..count {
        let (w, h) = (widths[i], heights[i]);
        let flags = r.u8()?;
        let column_major = flags & 1 != 0;
        let index_of = |n: usize| if column_major { (n % h) * w + n / h } else { n };
        let mut indexes = vec![0u8; w * h];
        for n in 0..w * h {
            indexes[index_of(n)] = r.u8()?;
        }
        let mut alpha = vec![0xffu8; w * h];
        if flags & 2 != 0 {
            for n in 0..w * h {
                alpha[index_of(n)] = r.u8()?;
            }
        }
        let pixels = indexes
            .iter()
            .zip(&alpha)
            .map(|(&idx, &a)| if idx == 0 { 0 } else { (a as u32) << 24 | palette[idx as usize] & 0xff_ffff })
            .collect();
        sprites.push(Sprite { min_x: min_x[i] as i32, min_y: min_y[i] as i32, frame_width, frame_height, width: w, height: h, pixels });
    }
    Ok(sprites)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a one-sprite archive: 2×2, column-major, palette [_, red, blue].
    fn sample() -> Vec<u8> {
        let mut d = vec![1u8, 1, 0, 2, 1]; // flags=column-major; pixels by column: (0,0)=1 (0,1)=0 (1,0)=2 (1,1)=1
        d.extend([0xff, 0, 0, 0, 0, 0xff]); // palette entries 1 and 2
        d.extend([0, 2, 0, 2, 2]); // max w, max h, palette size - 1
        d.extend([0, 3, 0, 4, 0, 2, 0, 2]); // min x, min y, width, height
        d.extend([0, 1]); // count
        d
    }

    #[test]
    fn decodes_column_major_with_transparency() {
        let s = &decode(&sample()).unwrap()[0];
        assert_eq!((s.min_x, s.min_y, s.width, s.height), (3, 4, 2, 2));
        assert_eq!(s.pixels, vec![0xffff0000, 0xff0000ff, 0, 0xffff0000]);
    }
}
