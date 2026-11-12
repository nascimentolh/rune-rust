//! Loading-screen definitions (index 33). Port of `GameTipsLoader` and the element
//! decoders called from `Class393.method6745`.
//!
//! Archive 0 is the header: version (≥ 4) · element type count · each type's tag ·
//! group count · highest group id · [version > 2: lead screen u16 (-1 = none), its time u24,
//! its fade u16] · per group: id u8 · shuffle u8 · entry count u16 · entries (screen u16,
//! time ms u24, fade ms u16).
//! Every other archive is one screen: element count u8, then per element its type u8 and fields.

use super::buffer::Reader;
use super::error::{corrupt, Result};

/// Tag per element type that the client expects in the header (`Class112.method1870`).
const TYPE_TAGS: [u8; 10] = [1, 2, 2, 2, 1, 1, 1, 2, 1, 2];
const MIN_VERSION: u8 = 4;

/// One screen in a group's running order.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub screen: u32,
    /// How long it stays before the next one may replace it.
    pub time_ms: u32,
    /// How long the crossfade away from it lasts.
    pub fade_ms: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub shuffle: bool,
    pub entries: Vec<Entry>,
}

pub struct Header {
    /// By group id; groups the file doesn't list are empty.
    pub groups: Vec<Group>,
    /// After the last screen, start again from the first (`method6790`).
    pub has_lead_screen: bool,
}

impl Header {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let version = if data.is_empty() { 0 } else { r.u8()? };
        if version < MIN_VERSION {
            return Ok(Header { groups: Vec::new(), has_lead_screen: false });
        }
        let type_count = r.u8()? as usize;
        let tags = r.bytes(type_count)?;
        if tags != TYPE_TAGS {
            return Ok(Header { groups: Vec::new(), has_lead_screen: false });
        }
        let group_count = r.u8()?;
        let highest_id = r.u8()? as usize;
        let (lead, lead_time, lead_fade) = if version > 2 { (r.i16()? as i32, r.u24()?, r.u16()? as u32) } else { (-1, 0, 0) };
        let entry =
            |r: &mut Reader| -> Result<Entry> { Ok(Entry { screen: r.u16()? as u32, time_ms: r.u24()?, fade_ms: r.u16()? as u32 }) };
        let lead_entry = (lead != -1).then_some(Entry { screen: lead as u32, time_ms: lead_time, fade_ms: lead_fade });

        let mut groups: Vec<Option<Group>> = vec![None; highest_id + 1];
        for _ in 0..group_count {
            let id = r.u8()? as usize;
            let shuffle = r.u8()? == 1;
            let n = r.u16()?;
            let mut entries: Vec<Entry> = lead_entry.iter().cloned().collect();
            for _ in 0..n {
                entries.push(entry(&mut r)?);
            }
            if id >= groups.len() {
                return corrupt(format!("loading screen group {id} above the highest id {highest_id}"));
            }
            groups[id] = Some(Group { shuffle, entries });
        }
        let groups = groups
            .into_iter()
            .map(|g| g.unwrap_or_else(|| Group { shuffle: false, entries: lead_entry.iter().cloned().collect() }))
            .collect();
        Ok(Header { groups, has_lead_screen: lead != -1 })
    }

    /// The running order for a group (`method6786`); unknown groups give nothing.
    /// ponytail: shuffled groups keep file order (the default group isn't shuffled in this cache);
    /// add the client's Java-Random shuffle if a shuffled group is ever used.
    pub fn order(&self, group: usize) -> &[Entry] {
        self.groups.get(group).map_or(&[], |g| &g.entries)
    }
}

/// Where an element sits: left/top, centre, or right/bottom of the screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anchor {
    Start,
    Centre,
    End,
}

impl Anchor {
    fn decode(r: &mut Reader) -> Result<Self> {
        match r.u8()? {
            0 => Ok(Anchor::Start),
            1 => Ok(Anchor::Centre),
            2 => Ok(Anchor::End),
            other => corrupt(format!("unknown anchor {other}")),
        }
    }

    /// Port of `Class356.method6221` / `Class353.method6198`.
    pub fn offset(self, size: i32, screen: i32) -> i32 {
        match self {
            Anchor::Start => 0,
            Anchor::Centre => (screen - size) / 2,
            Anchor::End => screen - size,
        }
    }
}

/// Fields shared by the bar-like elements (`Class383.method6512`).
#[derive(Clone, Debug, PartialEq)]
pub struct BarBox {
    pub h_anchor: Anchor,
    pub v_anchor: Anchor,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    /// Added to the vertical position of the status text.
    pub text_offset: i32,
    pub font: i32,
    pub text_colour: u32,
}

impl BarBox {
    fn decode(r: &mut Reader) -> Result<Self> {
        Ok(BarBox {
            h_anchor: Anchor::decode(r)?,
            v_anchor: Anchor::decode(r)?,
            x: r.i16()? as i32,
            y: r.i16()? as i32,
            width: r.u16()? as i32,
            height: r.u16()? as i32,
            text_offset: r.i16()? as i32,
            font: r.big_smart_or_none()?,
            text_colour: r.i32()? as u32,
        })
    }
}

/// The six images of a progress bar (`Class350_Sub3`).
#[derive(Clone, Debug, PartialEq)]
pub struct BarImages {
    pub fill: i32,
    pub empty: i32,
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    /// Type 5 (`Class357`): an image from index 32.
    Image { image: i32, h_anchor: Anchor, v_anchor: Anchor, x: i32, y: i32 },
    /// Type 7 (`Class62`): a paragraph in a box.
    Text {
        text: String,
        h_anchor: Anchor,
        v_anchor: Anchor,
        x: i32,
        y: i32,
        h_align: u8,
        v_align: u8,
        line_height: i32,
        width: i32,
        height: i32,
        font: i32,
        colour: u32,
        shadow: i32,
    },
    /// Types 3 and 9 (`Class350_Sub3`, `_Sub1`): an image-built progress bar; type 9's fill scrolls.
    ImageBar { bar: BarBox, images: BarImages, scroll_speed: i32 },
    /// Types 0, 1, 2, 4, 6 and 8: decoded so the stream stays aligned, but none of the
    /// loading screens in the cache use them, so their renderers aren't ported yet.
    Unported { kind: u8 },
}

/// One loading screen: elements drawn in order over black.
#[derive(Clone, Debug, PartialEq)]
pub struct Screen {
    pub elements: Vec<Element>,
}

impl Screen {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let n = r.u8()?;
        let elements = (0..n).map(|_| decode_element(&mut r)).collect::<Result<_>>()?;
        Ok(Screen { elements })
    }

    /// Every image id the screen draws (index 32).
    pub fn images(&self) -> Vec<i32> {
        self.elements
            .iter()
            .flat_map(|e| match e {
                Element::Image { image, .. } => vec![*image],
                Element::ImageBar { images: i, .. } => vec![i.fill, i.empty, i.left, i.right, i.top, i.bottom],
                _ => vec![],
            })
            .collect()
    }

    /// Every font id the screen uses (index 32 glyphs + index 13 metrics).
    pub fn fonts(&self) -> Vec<i32> {
        self.elements
            .iter()
            .filter_map(|e| match e {
                Element::Text { font, .. } => Some(*font),
                Element::ImageBar { bar, .. } => Some(bar.font),
                _ => None,
            })
            .collect()
    }
}

fn bar_images(r: &mut Reader) -> Result<BarImages> {
    Ok(BarImages {
        fill: r.big_smart_or_none()?,
        empty: r.big_smart_or_none()?,
        left: r.big_smart_or_none()?,
        right: r.big_smart_or_none()?,
        top: r.big_smart_or_none()?,
        bottom: r.big_smart_or_none()?,
    })
}

fn decode_element(r: &mut Reader) -> Result<Element> {
    let kind = r.u8()?;
    Ok(match kind {
        // Type 0: a colour (int).
        0 => {
            r.i32()?;
            Element::Unported { kind }
        }
        // Types 1 and 2: plain bars (box + two ints, type 2 adds an image).
        1 | 2 => {
            BarBox::decode(r)?;
            r.i32()?;
            r.i32()?;
            if kind == 2 {
                r.big_smart_or_none()?;
            }
            Element::Unported { kind }
        }
        3 | 9 => {
            let bar = BarBox::decode(r)?;
            let images = bar_images(r)?;
            let scroll_speed = if kind == 9 { r.i16()? as i32 } else { 0 };
            Element::ImageBar { bar, images, scroll_speed }
        }
        // Type 4: news box (u8, anchors, position, size, three ints, flag).
        4 => {
            r.u8()?;
            Anchor::decode(r)?;
            Anchor::decode(r)?;
            r.bytes(8)?;
            r.bytes(12)?;
            r.u8()?;
            Element::Unported { kind }
        }
        5 | 6 => {
            let image = r.big_smart_or_none()?;
            let (h_anchor, v_anchor) = (Anchor::decode(r)?, Anchor::decode(r)?);
            let (x, y) = (r.i16()? as i32, r.i16()? as i32);
            if kind == 6 {
                // Tinted image: tint colour (s24).
                r.i24()?;
                Element::Unported { kind }
            } else {
                Element::Image { image, h_anchor, v_anchor, x, y }
            }
        }
        7 => Element::Text {
            text: r.string()?,
            h_anchor: Anchor::decode(r)?,
            v_anchor: Anchor::decode(r)?,
            x: r.i16()? as i32,
            y: r.i16()? as i32,
            h_align: r.u8()?,
            v_align: r.u8()?,
            line_height: r.u8()? as i32,
            width: r.u16()? as i32,
            height: r.u16()? as i32,
            font: r.big_smart_or_none()?,
            colour: r.i32()? as u32,
            shadow: r.i32()?,
        },
        // Type 8: a full-screen image id.
        8 => {
            r.big_smart_or_none()?;
            Element::Unported { kind }
        }
        other => return corrupt(format!("unknown loading screen element type {other}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_with_one_group() {
        let mut d = vec![4, 10];
        d.extend(TYPE_TAGS);
        d.extend([1, 1, 0xff, 0xff, 0, 0, 0, 0, 0]); // 1 group, highest id 1, no lead screen
        d.extend([1, 0, 0, 2]); // group 1, not shuffled, 2 entries
        d.extend([0, 2, 0, 0, 0, 0x01, 0xf4]); // screen 2, 0 ms, fade 500
        d.extend([0, 4, 0, 0x23, 0x28, 0x03, 0xe8]); // screen 4, 9000 ms, fade 1000
        let h = Header::decode(&d).unwrap();
        assert_eq!(h.order(1), &[Entry { screen: 2, time_ms: 0, fade_ms: 500 }, Entry { screen: 4, time_ms: 9000, fade_ms: 1000 }]);
        assert!(h.order(0).is_empty());
        assert!(!h.has_lead_screen);
    }

    #[test]
    fn image_and_text_elements() {
        let mut d = vec![2, 5, 0x0e, 0xb3, 1, 0, 0xff, 0x70, 0, 25];
        d.extend([7]);
        d.extend(b"Hi\0");
        d.extend([1, 0, 0xff, 0x31, 0, 0xef, 0, 2, 19, 1, 0x22, 0, 0xe8, 0x0e, 0xd2, 0xff, 0x33, 0x33, 0x33, 0, 0, 0, 0]);
        let s = Screen::decode(&d).unwrap();
        assert_eq!(s.elements[0], Element::Image { image: 3763, h_anchor: Anchor::Centre, v_anchor: Anchor::Start, x: -144, y: 25 });
        assert!(matches!(&s.elements[1], Element::Text { text, font: 3794, colour: 0xff33_3333, line_height: 19, .. } if text == "Hi"));
    }
}
