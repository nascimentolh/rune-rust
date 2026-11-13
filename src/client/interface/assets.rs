//! What interfaces need from the cache, fetched on demand and kept: interface archives
//! (index 3), CS2 scripts (index 12), sprites (index 8) and fonts (glyphs in index 8,
//! metrics in index 13), as `Class378.method6426` pairs them.

use crate::cache::cs2::Script;
use crate::cache::enums::{Enum, EnumValue};
use crate::cache::font_metrics::FontMetrics;
use crate::cache::params::{decode_struct, Param};
use crate::cache::sprite::{self, Sprite};
use crate::client::font::Font;
use crate::client::source::{Archives, Files};
use std::collections::HashMap;
use std::rc::Rc;

const CONFIGS: u8 = 2;
/// `SharedConfigsType.PARAMS` / `STRUCTS`: archives of index 2.
const PARAMS: i32 = 11;
const STRUCTS: i32 = 26;
const SPRITES: u8 = 8;
const SCRIPTS: u8 = 12;
const ENUMS: u8 = 17;
const VARBITS: u8 = 22;
const FONT_METRICS: u8 = 13;
/// `DEFAULTS_INDEX` and its `DefaultsFile.ENTITY` file.
const DEFAULTS: u8 = 28;
const ENTITY_DEFAULTS: i32 = 3;

/// A sprite as a component shows it: the key of `IComponentDefinitions.method2048`'s cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Look {
    pub sprite: i32,
    pub h_flip: bool,
    pub v_flip: bool,
    pub border: i32,
    pub shadow: i32,
}

pub struct Assets {
    archives: Archives,
    scripts: HashMap<i32, Option<Rc<Script>>>,
    sprites: HashMap<Look, Option<Rc<Sprite>>>,
    fonts: HashMap<i32, Option<Rc<Font>>>,
    enums: HashMap<i32, Rc<Enum>>,
    metrics: HashMap<i32, Option<Rc<FontMetrics>>>,
}

impl Assets {
    pub fn new(archives: Archives) -> Self {
        Assets {
            archives,
            scripts: HashMap::new(),
            sprites: HashMap::new(),
            fonts: HashMap::new(),
            enums: HashMap::new(),
            metrics: HashMap::new(),
        }
    }

    pub fn archive(&mut self, index: u8, archive: u32) -> Option<Rc<Files>> {
        self.archives.archive(index, archive)
    }

    pub fn archive_count(&mut self, index: u8) -> Option<usize> {
        self.archives.archive_count(index)
    }

    /// Region (`rx`, `ry`)'s terrain, `m{x}_{y}` in index 5, `None` where there is none.
    pub fn terrain(&mut self, rx: i32, ry: i32) -> Option<crate::cache::map_terrain::Terrain> {
        use crate::cache::map_terrain::{archive_name, name_hash, Terrain};
        let archive = self.archives.archive_named(5, name_hash(&archive_name(true, rx, ry)))?;
        let data = self.first_file(5, archive as i32)?;
        Terrain::decode(&data, rx, ry).map_err(|e| eprintln!("terrain {rx},{ry}: {e}")).ok()
    }

    /// Underlay `id` (config archive 1).
    pub fn underlay(&mut self, id: u32) -> Option<crate::cache::floors::Underlay> {
        let data = self.file(2, 1, id)?;
        crate::cache::floors::Underlay::decode(&data).ok()
    }

    /// Overlay `id` (config archive 4).
    pub fn overlay(&mut self, id: u32) -> Option<crate::cache::floors::Overlay> {
        let data = self.file(2, 4, id)?;
        crate::cache::floors::Overlay::decode(&data).ok()
    }

    /// The login music (`Class260.anInt3228`): the first opcode-1 track of defaults file 4.
    pub fn login_track(&mut self) -> Option<u32> {
        let data = self.first_file(DEFAULTS, 4)?;
        crate::cache::music::login_track(&data).ok().flatten()
    }

    /// Song `track` of `index` (6 songs, 11 jingles) with what it plays, ready to decode
    /// (`Node_Sub7.method12164`, `Class250`).
    pub fn song(&mut self, index: u8, track: u32) -> Result<crate::client::music::load::Raw, String> {
        let packed = self.first_file(index, track as i32).ok_or(format!("track {track} isn't in the cache"))?;
        let midi = crate::cache::music::to_midi(&packed).map_err(|e| format!("track {track}: {e}"))?;
        let single_synth = self.archive_count(4) == Some(1);
        crate::client::music::load::gather(&mut |index, archive| self.archive(index, archive), single_synth, midi)
    }

    /// `Index.getFile(archive, file)`.
    pub fn file(&mut self, index: u8, archive: i32, file: u32) -> Option<Vec<u8>> {
        self.archives.archive(index, u32::try_from(archive).ok()?)?.get(file as usize)?.clone()
    }

    fn first_file(&mut self, index: u8, archive: i32) -> Option<Vec<u8>> {
        self.file(index, archive, 0)
    }

    /// `QuickchatFiller.ENTITY_DEFAULTS`.
    pub fn defaults(&mut self) -> Result<crate::cache::entity_defaults::Defaults, String> {
        let data = self.first_file(DEFAULTS, ENTITY_DEFAULTS).ok_or("the entity defaults aren't in the cache")?;
        crate::cache::entity_defaults::decode(&data).map_err(|e| e.to_string())
    }

    /// `ENTITY_DEFAULTS.loginWindow`: the interface the login screen shows.
    pub fn login_window(&mut self) -> Result<u32, String> {
        self.defaults()?.login_window.ok_or_else(|| "no login window in the entity defaults".into())
    }

    /// `VarBitIndexLoader.getVarBit`: group id >> 10, file id & 1023 of index 22.
    pub fn varbit(&mut self, id: i32) -> Result<crate::cache::varbit::VarBit, String> {
        let data = self.file(VARBITS, id >> 10, id as u32 & 0x3ff).ok_or(format!("varbit {id} isn't in the cache"))?;
        crate::cache::varbit::decode(&data).map_err(|e| e.to_string())
    }

    /// `RSMesh.decodeMesh(MESH_INDEX, id)`: index 7, file 0 of archive `id`.
    pub fn mesh(&mut self, id: u32) -> Option<crate::cache::mesh::Mesh> {
        let data = self.first_file(7, id as i32)?;
        crate::cache::mesh::Mesh::decode(&data).map_err(|e| eprintln!("mesh {id}: {e}")).ok()
    }

    /// `AsyncInputStream.getCS2Script`.
    pub fn script(&mut self, id: i32) -> Option<Rc<Script>> {
        if !self.scripts.contains_key(&id) {
            let script = self.first_file(SCRIPTS, id).and_then(|data| match Script::decode(&data) {
                Ok(s) => Some(Rc::new(s)),
                Err(e) => {
                    eprintln!("script {id}: {e}");
                    None
                }
            });
            self.scripts.insert(id, script);
        }
        self.scripts[&id].clone()
    }

    /// `EnumIndexLoader.getEnumDefinitions`: a missing file gives an empty enum.
    pub fn enumeration(&mut self, id: i32) -> Result<Rc<Enum>, String> {
        if let Some(e) = self.enums.get(&id) {
            return Ok(Rc::clone(e));
        }
        let e = match self.file(ENUMS, ((id as u32) >> 8) as i32, id as u32 & 0xff) {
            Some(data) => Enum::decode(&data).map_err(|e| format!("enum {id}: {e}"))?,
            None => Enum::default(),
        };
        let e = Rc::new(e);
        self.enums.insert(id, Rc::clone(&e));
        Ok(e)
    }

    /// `STRUCT_PARAM`: the struct's value for a param, or the param's default
    /// (`StructIndexLoader.getStruct`, `ParamIndexLoader.getParam`; missing files are empty).
    pub fn struct_param(&mut self, id: i32, param: i32) -> Result<EnumValue, String> {
        let p = match self.file(CONFIGS, PARAMS, param as u32) {
            Some(d) => Param::decode(&d).map_err(|e| format!("param {param}: {e}"))?,
            None => Param::default(),
        };
        let values = match (id >= 0).then(|| self.file(CONFIGS, STRUCTS, id as u32)).flatten() {
            Some(d) => decode_struct(&d).map_err(|e| format!("struct {id}: {e}"))?,
            None => Default::default(),
        };
        Ok(match (p.is_string(), values.get(&param)) {
            (true, Some(EnumValue::Text(t))) => EnumValue::Text(t.clone()),
            (true, _) => EnumValue::Text(p.default_text.unwrap_or_else(|| "null".into())),
            (false, Some(EnumValue::Int(v))) => EnumValue::Int(*v),
            (false, _) => EnumValue::Int(p.default_int),
        })
    }

    /// `method2048`: frame 0 of the sprite, flipped, outlined and shadowed as asked.
    pub fn sprite(&mut self, look: Look) -> Option<Rc<Sprite>> {
        if !self.sprites.contains_key(&look) {
            let sprite =
                self.first_file(SPRITES, look.sprite).and_then(|d| sprite::decode(&d).ok()).and_then(|s| s.into_iter().next()).map(|s| {
                    let mut s = s;
                    dress(&mut s, look);
                    Rc::new(s)
                });
            self.sprites.insert(look, sprite);
        }
        self.sprites[&look].clone()
    }

    /// `Class378.method6426`: glyph sprites and metrics share the font's id.
    pub fn font(&mut self, id: i32) -> Option<Rc<Font>> {
        if !self.fonts.contains_key(&id) {
            let glyphs = self.first_file(SPRITES, id).and_then(|d| sprite::decode(&d).ok());
            let metrics = self.first_file(FONT_METRICS, id).and_then(|d| FontMetrics::decode(&d).ok());
            let font = glyphs.zip(metrics).map(|(g, m)| Rc::new(Font::new(m, g)));
            self.fonts.insert(id, font);
        }
        self.fonts[&id].clone()
    }

    /// `Class94.getFontMetrics`: a font's metrics alone (index 13), for measuring text.
    pub fn metrics(&mut self, font: i32) -> Option<Rc<FontMetrics>> {
        if !self.metrics.contains_key(&font) {
            let m = self.first_file(FONT_METRICS, font).and_then(|d| FontMetrics::decode(&d).ok()).map(Rc::new);
            self.metrics.insert(font, m);
        }
        self.metrics[&font].clone()
    }
}

/// The edits of `method2048`, in its order: `method1526` (flip vertically), `method1525`
/// (flip horizontally), `method1554` (pad inside the frame), `method1536` (outline, once
/// per border pixel) and `method1524` (drop shadow).
fn dress(s: &mut Sprite, look: Look) {
    if look.v_flip {
        let rows: Vec<Vec<u32>> = s.pixels.chunks(s.width.max(1)).rev().map(<[u32]>::to_vec).collect();
        s.pixels = rows.concat();
        s.min_y = s.frame_height - s.min_y - s.height as i32;
    }
    if look.h_flip {
        for row in s.pixels.chunks_mut(s.width.max(1)) {
            row.reverse();
        }
        s.min_x = s.frame_width - s.min_x - s.width as i32;
    }
    if look.border > 0 {
        pad(s, look.border);
    } else if look.shadow != 0 {
        pad(s, 1);
    }
    if look.border >= 1 {
        outline(s, 0x00_0001);
    }
    if look.border >= 2 {
        outline(s, 0xff_ffff);
    }
    if look.shadow != 0 {
        shadow(s, look.shadow as u32 & 0xff_ffff);
    }
}

/// `method1554`: grows the image by up to `p` pixels on each side, inside its frame.
fn pad(s: &mut Sprite, p: i32) {
    let (w, h) = (s.width as i32, s.height as i32);
    if s.frame_width == w && s.frame_height == h {
        return;
    }
    let left = p.min(s.min_x);
    let right = if p + s.min_x + w > s.frame_width { s.frame_width - s.min_x - w } else { p };
    let top = p.min(s.min_y);
    let bottom = if p + s.min_y + h > s.frame_height { s.frame_height - s.min_y - h } else { p };
    let (nw, nh) = (left + right + w, top + bottom + h);
    let mut pixels = vec![0u32; (nw * nh).max(0) as usize];
    for y in 0..h {
        for x in 0..w {
            pixels[((y + top) * nw + x + left) as usize] = s.pixels[(y * w + x) as usize];
        }
    }
    s.min_x -= left;
    s.min_y -= top;
    (s.width, s.height, s.pixels) = (nw as usize, nh as usize, pixels);
}

/// `method1536`: every clear pixel next to (left, up, right, down) a solid one takes `rgb`.
fn outline(s: &mut Sprite, rgb: u32) {
    let (w, h) = (s.width, s.height);
    let solid = |p: &[u32], i: usize| p[i] != 0;
    let old = s.pixels.clone();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if old[i] == 0
                && ((x > 0 && solid(&old, i - 1))
                    || (y > 0 && solid(&old, i - w))
                    || (x + 1 < w && solid(&old, i + 1))
                    || (y + 1 < h && solid(&old, i + w)))
            {
                s.pixels[i] = 0xff00_0000 | rgb;
            }
        }
    }
}

/// `method1524`: every clear pixel whose up-left neighbour is solid takes the shadow colour.
fn shadow(s: &mut Sprite, rgb: u32) {
    let w = s.width;
    for y in (1..s.height).rev() {
        for x in (1..w).rev() {
            let i = y * w + x;
            if s.pixels[i] == 0 && s.pixels[i - 1 - w] != 0 {
                s.pixels[i] = 0xff00_0000 | rgb;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot() -> Sprite {
        Sprite { min_x: 1, min_y: 0, frame_width: 4, frame_height: 2, width: 1, height: 1, pixels: vec![0xffff_0000] }
    }

    #[test]
    fn flips_move_the_image_inside_its_frame() {
        let mut s = dot();
        dress(&mut s, Look { sprite: 0, h_flip: true, v_flip: true, border: 0, shadow: 0 });
        assert_eq!((s.min_x, s.min_y), (2, 1));
    }

    #[test]
    fn shadow_pads_then_marks_down_right() {
        let mut s = dot();
        dress(&mut s, Look { sprite: 0, h_flip: false, v_flip: false, border: 0, shadow: 0x123456 });
        assert_eq!((s.min_x, s.min_y, s.width, s.height), (0, 0, 3, 2));
        assert_eq!(s.pixels[3 + 2], 0xff12_3456);
    }
}
