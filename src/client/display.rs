//! The full-screen sizes the Options panel lists. Port of `ClanVarDefinitions.method6825`.
//!
//! The reference asks Java for the monitor's display modes (`GraphicsDevice.getDisplayModes`).
//! That needs the system's mode list, which this client can't read without `unsafe` code;
//! instead it takes the usual PC sizes up to the main monitor's own size (read with
//! `display-info`), plus that size, as the modes. Then the reference's filter: at least
//! 800 × 600, at most 800 × 600 with "max screen size" 2 and 1024 × 768 with 1, one entry per
//! size, sorted by area (smallest first).

/// Common monitor sizes, the stand-in for the system's mode list.
const USUAL: [(i32, i32); 22] = [
    (800, 600),
    (1024, 768),
    (1152, 864),
    (1280, 720),
    (1280, 768),
    (1280, 800),
    (1280, 960),
    (1280, 1024),
    (1360, 768),
    (1366, 768),
    (1440, 900),
    (1600, 900),
    (1600, 1200),
    (1680, 1050),
    (1920, 1080),
    (1920, 1200),
    (2048, 1152),
    (2560, 1080),
    (2560, 1440),
    (2560, 1600),
    (3440, 1440),
    (3840, 2160),
];

/// The main monitor's size in pixels, if the system tells.
pub fn monitor() -> Option<(i32, i32)> {
    let all = display_info::DisplayInfo::all().ok()?;
    let main = all.iter().find(|d| d.is_primary).or(all.first())?;
    Some((main.width as i32, main.height as i32))
}

/// `method6825` for a monitor of `size` and the "max screen size" preference.
pub fn modes(size: Option<(i32, i32)>, max_screen_size: i32) -> Vec<(i32, i32)> {
    let Some((mw, mh)) = size else { return Vec::new() };
    let mut out: Vec<(i32, i32)> = Vec::new();
    for (w, h) in USUAL.into_iter().filter(|&(w, h)| w <= mw && h <= mh).chain([(mw, mh)]) {
        let fits =
            w >= 800 && h >= 600 && (max_screen_size != 2 || (w <= 800 && h <= 600)) && (max_screen_size != 1 || (w <= 1024 && h <= 768));
        if fits && !out.contains(&(w, h)) {
            out.push((w, h));
        }
    }
    out.sort_by_key(|&(w, h)| w * h);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_up_to_the_monitor_smallest_first() {
        let m = modes(Some((1920, 1080)), 0);
        assert_eq!(m.first(), Some(&(800, 600)));
        assert_eq!(m.last(), Some(&(1920, 1080)));
        assert!(m.iter().all(|&(w, h)| w <= 1920 && h <= 1080));
        assert_eq!(modes(Some((1920, 1080)), 1).last(), Some(&(1024, 768)));
        assert_eq!(modes(Some((1920, 1080)), 2), vec![(800, 600)]);
        assert!(modes(None, 0).is_empty());
    }
}
