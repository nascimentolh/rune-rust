//! The country flags of the world list, read from the cache so the administrator can pick one
//! by looking at it instead of by its number.
//!
//! A world's country is a key of enum 1810, which the World Select turns into the sprite it
//! draws beside the location's name (script 3117: `ENUM(…, 1810, country)` then
//! `IF_SETGRAPHIC`). The flags are read once, when the server starts, and kept as pixels; the
//! status window turns them into pictures as it first draws them.

use crate::cache::enums::Enum;
use crate::cache::{sprite, Cache};

/// `IndexReference.INDEX_ENUMS` (17: archive id >> 8, file id & 0xff) and
/// `INDEX_SPRITES` (8: one archive per sprite).
const ENUMS: u8 = 17;
const SPRITES: u8 = 8;
/// The enum the World Select looks a world's country up in.
const COUNTRY_FLAGS: i32 = 1810;

/// One country's flag.
pub struct Flag {
    /// The number a world's `country` is set to.
    pub country: i32,
    pub width: u16,
    pub height: u16,
    /// Four bytes a pixel, row by row.
    pub rgba: Vec<u8>,
}

/// Every country that has a flag in this cache, smallest number first. Empty when the cache
/// has no such enum (an older or cut-down cache): the panel then only takes a number.
pub fn load(cache: &mut Cache) -> Vec<Flag> {
    let Ok(files) = cache.read_files(ENUMS, (COUNTRY_FLAGS as u32) >> 8) else { return Vec::new() };
    let Some(Ok(flags)) = files.get(COUNTRY_FLAGS as usize & 0xff).map(|d| Enum::decode(d)) else { return Vec::new() };
    flags.keys().iter().filter_map(|&country| one(cache, &flags, country)).collect()
}

/// The flag of one country, if the enum has a sprite for it and the sprite reads.
fn one(cache: &mut Cache, flags: &Enum, country: i32) -> Option<Flag> {
    let sprite = flags.int(country).ok().filter(|&id| id > 0)?;
    let files = cache.read_files(SPRITES, sprite as u32).ok()?;
    let image = sprite::decode(files.first()?).ok()?.into_iter().next()?;
    // `Sprite.pixels` is ARGB, alpha 0 where nothing was drawn.
    let rgba = image.pixels.iter().flat_map(|&p| [(p >> 16) as u8, (p >> 8) as u8, p as u8, (p >> 24) as u8]).collect();
    Some(Flag { country, width: image.width as u16, height: image.height as u16, rgba })
}
