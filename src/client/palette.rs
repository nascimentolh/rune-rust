//! The client's colour table: packed HSL (6 bits hue, 3 saturation, 7 lightness) to 24-bit RGB.
//! Port of `CutsceneCameraMovement.method1365` (`Class540.anIntArray7136`): standard HSL to RGB
//! with hue + 1/128 and saturation + 1/16, then each channel raised to the power of a
//! brightness the client picks once at random, 0.7 ± 0.015.

/// `0.7D + (Math.random() * 0.03D - 0.015D)`.
pub fn random_brightness() -> f64 {
    0.7 + (rand::random::<f64>() * 0.03 - 0.015)
}

/// One HSL channel (`t` the hue shifted by ±1/3).
fn channel(t: f64, p: f64, q: f64) -> f64 {
    if 6.0 * t < 1.0 {
        q + (p - q) * 6.0 * t
    } else if 2.0 * t < 1.0 {
        p
    } else if 3.0 * t < 2.0 {
        q + 6.0 * (0.6666666666666666 - t) * (p - q)
    } else {
        q
    }
}

/// The RGB of packed HSL `hsl` at `brightness`.
pub fn hsl_to_rgb(hsl: u16, brightness: f64) -> i32 {
    let i = hsl as i32;
    let hue = ((i >> 10) & 0x3f) as f64 / 64.0 + 0.0078125;
    let sat = 0.0625 + ((i >> 7) & 0x7) as f64 / 8.0;
    let light = (i & 0x7f) as f64 / 128.0;
    // Saturation is never 0 here (it starts at 1/16), so the colour always goes through HSL.
    let p = if light < 0.5 { light * (sat + 1.0) } else { sat + light - light * sat };
    let q = light * 2.0 - p;
    let mut red_hue = 0.3333333333333333 + hue;
    if red_hue > 1.0 {
        red_hue -= 1.0;
    }
    let mut blue_hue = hue - 0.3333333333333333;
    if blue_hue < 0.0 {
        blue_hue += 1.0;
    }
    let (r, g, b) = (channel(red_hue, p, q), channel(hue, p, q), channel(blue_hue, p, q));
    let (r, g, b) = ((r.powf(brightness) * 256.0) as i32, (g.powf(brightness) * 256.0) as i32, (b.powf(brightness) * 256.0) as i32);
    (b + (g << 8) + (r << 16)) & 0xff_ffff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grey_and_red_come_out_right() {
        // Lightness 127, lowest saturation: nearly grey (the table adds 1/16 saturation and
        // 1/128 hue to every colour), so the channels stay close.
        let grey = hsl_to_rgb(127, 1.0);
        let (r, g, b) = (grey >> 16, (grey >> 8) & 0xff, grey & 0xff);
        assert!(r.max(g).max(b) - r.min(g).min(b) < 24 && g > 200);
        // Hue 0, full saturation, middle lightness: red ahead of green and blue.
        let red = hsl_to_rgb((7 << 7) + 64, 0.7);
        assert!(red >> 16 > (red >> 8) & 0xff && red >> 16 > red & 0xff);
    }
}
