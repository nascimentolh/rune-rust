//! Textures: the texture table (index 26, `ImageIndexLoader`'s constructor) and each texture's
//! procedural material (index 9, `MaterialDefinitions`): a graph of operations
//! (`MaterialProperty.decode`), each reading its own settings, wired to the operations it takes
//! its input from, with three outputs (opaque, translucent, combined).
//!
//! Every operation's settings are read so the file stays in step (`MaterialProp0` ... `39`);
//! what drawing needs is kept as the operations get ported.

use super::buffer::Reader;
use super::error::{corrupt, Result};

/// One texture's entry in the texture table.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureDetails {
    /// `isGroundMesh` (byte 0): drawn on the ground as a plain colour.
    pub ground_mesh: bool,
    /// `isHalfSize`: drawn at 64 × 64 instead of 128 × 128.
    pub half_size: bool,
    /// `skipTriangles`: faces with it aren't drawn.
    pub skip_triangles: bool,
    /// `brightness`, `shadowFactor` (signed bytes).
    pub brightness: i8,
    pub shadow: i8,
    /// `color`: the texture's average colour, packed HSL.
    pub colour: u16,
    /// `isBrickTile`.
    pub brick: bool,
    /// `blendType`.
    pub blend: u8,
}

/// `ImageIndexLoader`'s constructor: a u16 count, then each field for every texture that
/// exists, field by field.
pub fn texture_table(data: &[u8]) -> Result<Vec<Option<TextureDetails>>> {
    let mut r = Reader::new(data);
    let count = r.u16()? as usize;
    let mut t: Vec<Option<TextureDetails>> = (0..count).map(|_| Ok((r.u8()? == 1).then(TextureDetails::default))).collect::<Result<_>>()?;
    let t_ = &mut t;
    each(&mut r, t_, |d, b| d.ground_mesh = b == 0)?;
    each(&mut r, t_, |d, b| d.half_size = b == 1)?;
    each(&mut r, t_, |d, b| d.skip_triangles = b == 1)?;
    each(&mut r, t_, |d, b| d.brightness = b as i8)?;
    each(&mut r, t_, |d, b| d.shadow = b as i8)?;
    each(&mut r, t_, |_, _| {})?; // effectId
    each(&mut r, t_, |_, _| {})?; // effectParam1
    for d in t_.iter_mut().flatten() {
        d.colour = r.u16()?;
    }
    each(&mut r, t_, |_, _| {})?; // textureSpeedU
    each(&mut r, t_, |_, _| {})?; // textureSpeedV
    each(&mut r, t_, |_, _| {})?; // aBool2087
    each(&mut r, t_, |d, b| d.brick = b == 1)?;
    each(&mut r, t_, |_, _| {})?; // useMipmaps
    each(&mut r, t_, |_, _| {})?; // repeatS
    each(&mut r, t_, |_, _| {})?; // repeatT
    each(&mut r, t_, |_, _| {})?; // hdr
    each(&mut r, t_, |_, _| {})?; // combineMode
    for _ in t_.iter().flatten() {
        r.i32()?; // effectParam2
    }
    each(&mut r, t_, |d, b| d.blend = b)?;
    Ok(t)
}

/// Reads one byte for every texture that exists into `set`.
fn each(r: &mut Reader, t: &mut [Option<TextureDetails>], mut set: impl FnMut(&mut TextureDetails, u8)) -> Result<()> {
    for d in t.iter_mut().flatten() {
        set(d, r.u8()?);
    }
    Ok(())
}

/// One operation of a material.
#[derive(Clone, Debug, PartialEq)]
pub struct Op {
    /// `MaterialProperty.getById`: 0..=39.
    pub kind: u8,
    /// The operations it takes its inputs from.
    pub inputs: Vec<usize>,
    /// A sprite (39, 18) or texture (36) it reads, −1 none.
    pub image: i32,
    /// `noPalette`: one channel instead of three.
    pub mono: bool,
    /// What the operation does with its settings (only the ported ones are kept).
    pub params: Params,
}

/// The settings of the operations that are ported, made ready (`method12321`).
#[derive(Clone, Debug, PartialEq)]
pub enum Params {
    /// 0: one value (`anInt9860`, default 4096).
    Constant(i32),
    /// 1: one colour, channels × 16 (`anInt9871`, `anInt9870`, `anInt9869`).
    Colour([i32; 3]),
    /// 2, 3: the x or y position, 0..4096.
    Horizontal,
    Vertical,
    /// 5: a box blur of radius (x, y).
    Blur(i32, i32),
    /// 6: clamped to (low, high).
    Clamp(i32, i32),
    /// 7: two inputs combined by mode 1..=12 (`anInt9895`, default 6).
    Blend(i32),
    /// 8: a value curve, 257 entries over 0..4096 (`aShortArray9801`).
    Curve(Box<[i16; 257]>),
    /// 10: a colour ramp, 257 entries over 0..4096 (`anIntArray9823`).
    Ramp(Box<[i32; 257]>),
    /// 9: mirrored in x and / or y.
    Mirror(bool, bool),
    /// 17: hue, saturation and lightness shifted (`anInt9918`, `anInt9919`, `anInt9923`).
    Hsl([i32; 3]),
    /// 19: input 0 read at an offset of input 2 × strength along the angle of input 1.
    Warp(i32),
    /// 21: inputs 0 and 1 mixed by input 2.
    Mix,
    /// 22: 4096 − input.
    Invert,
    /// 30: input × span >> 12 + low.
    Range(i32, i32),
    /// 32: relief of input 0's slopes, scale and light direction.
    Emboss(i32, [i32; 3]),
    /// 34: gradient noise over octaves.
    Noise(Box<Noise>),
    /// 15: distances to the nearest jittered cell points.
    Cells(Box<Cells>),
    /// 36: another texture's pixels (`MaterialPropTexture`), the op's `image`.
    Texture,
    /// 39: a sprite's pixels, stretched over the texture (`MaterialPropSprite`).
    Sprite,
    /// 18: a sprite's pixels, repeated (`MaterialPropSpriteSub18`).
    TiledSprite,
    /// 13: a hash of each pixel's position (`MaterialProp13.method15428`).
    Hash,
    /// 20: input 0 shrunk and repeated, (across, down) times.
    Tile(i32, i32),
    /// Any operation not ported yet.
    Unported,
}

/// `MaterialProp34`, made ready by `method12321`.
#[derive(Clone, Debug, PartialEq)]
pub struct Noise {
    /// `aBool9726`: the sum is halved and centred on 2048 after the last octave.
    pub centred: bool,
    /// `anInt9725`, `anInt9735`: how many noise cells across and down.
    pub cells: (i32, i32),
    /// `aShortArray9730`, `aShortArray9731`: each octave's strength and frequency.
    pub amplitude: Vec<i16>,
    pub frequency: Vec<i16>,
    /// `aByteArray9732`: `Class316.method5588(anInt9733)`.
    pub perm: Box<[u8; 512]>,
}

/// `MaterialProp15`, made ready by `method12321`.
#[derive(Clone, Debug, PartialEq)]
pub struct Cells {
    /// `anInt9961`, `anInt9962`: how many cells across and down.
    pub cells: (i32, i32),
    /// `anInt9963`: 0 nearest, 1 second, 2 second − nearest, 3 third, 4 fourth distance.
    pub output: i32,
    /// `anInt9949`: 1 squared, 2 Manhattan, 3 Chebyshev, 4 and 5 rooted, else Euclidean.
    pub distance: i32,
    /// `aByteArray9965`: `Class316.method5588(anInt9959)`.
    pub perm: Box<[u8; 512]>,
    /// `aShortArray9966`: each cell point's (x, y) offset (`method15412`).
    pub offsets: Box<[i16; 512]>,
}

/// `java.util.Random`, which the noise operations seed with their seed setting.
struct JavaRandom(i64);

impl JavaRandom {
    const MASK: i64 = (1 << 48) - 1;

    fn new(seed: i64) -> Self {
        JavaRandom((seed ^ 0x5DEECE66D) & Self::MASK)
    }

    /// `nextInt()`.
    fn next_int(&mut self) -> i32 {
        self.0 = self.0.wrapping_mul(0x5DEECE66D).wrapping_add(0xB) & Self::MASK;
        (self.0 >> 16) as i32
    }

    /// `Class476.method7931`: 0..n (n > 0) without bias.
    fn below(&mut self, n: i32) -> i32 {
        // `CutsceneAction_Sub9.method14585`: n == n & -n, a power of two for n > 0.
        if (n as u32).is_power_of_two() {
            return ((self.next_int() as u32 as i64 * n as i64) >> 32) as i32;
        }
        let limit = i32::MIN.wrapping_sub((4294967296i64 % n as i64) as i32);
        loop {
            let v = self.next_int();
            if v < limit {
                // `Class302.method5360`.
                return (v.wrapping_add((v as u32 >> 31) as i32) % n) + ((v >> 31) & (n - 1));
            }
        }
    }
}

/// `Class316.method5588`: the shuffled byte table of a seed. Entries 255 and 256 stay 0, as
/// the reference leaves them.
fn permutation(seed: i32) -> Box<[u8; 512]> {
    let mut b = Box::new([0u8; 512]);
    for (i, v) in b.iter_mut().take(255).enumerate() {
        *v = i as u8;
    }
    let mut random = JavaRandom::new(seed as i64);
    for i in 0..255 {
        let j = 255 - i;
        let k = random.below(j as i32) as usize;
        let t = b[k];
        b[k] = b[j];
        b[j] = t;
        b[511 - i] = t;
    }
    b
}

/// `MaterialProperty(numParams, noPalette)`: which kinds start with one channel.
const MONO: [bool; 40] = [
    true, false, true, true, true, false, false, false, true, false, false, false, true, true, true, true, true, false, false, false,
    false, false, false, false, true, false, true, true, true, true, false, true, true, false, true, true, false, true, true, false,
];

#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub ops: Vec<Op>,
    /// `opaqueProperty`, `translucentProperty`, `combinedProperty`.
    pub outputs: [usize; 3],
}

/// How many inputs each operation takes (`MaterialProperty(numParams, ...)`).
const INPUTS: [usize; 40] =
    [0, 0, 0, 0, 0, 1, 1, 2, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 3, 1, 3, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 0, 0];

impl Material {
    /// `MaterialDefinitions(ByteBuf)`.
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let count = r.u8()? as usize;
        let mut ops = Vec::with_capacity(count);
        for _ in 0..count {
            r.u8()?;
            let kind = r.u8()?;
            if kind >= 40 {
                return corrupt(format!("material operation {kind}"));
            }
            r.u8()?; // anInt7668: the palette size
            let mut image = -1;
            let mut op = Settings::new(kind);
            let settings = r.u8()?;
            for _ in 0..settings {
                let setting = r.u8()?;
                if op.read(&mut r, kind, setting)? {
                    continue;
                }
                if let Some(id) = skip_setting(&mut r, kind, setting)? {
                    image = id;
                }
            }
            let inputs = (0..INPUTS[kind as usize]).map(|_| Ok(r.u8()? as usize)).collect::<Result<Vec<_>>>()?;
            let (mono, params) = op.finish(kind)?;
            ops.push(Op { kind, inputs, image, mono, params });
        }
        let outputs = [r.u8()? as usize, r.u8()? as usize, r.u8()? as usize];
        if ops.iter().flat_map(|o| &o.inputs).chain(&outputs).any(|&i| i >= ops.len()) {
            return corrupt("material input out of range".to_string());
        }
        Ok(Material { ops, outputs })
    }
}

/// One operation's settings while it is read.
struct Settings {
    mono: bool,
    /// Plain values in the order each kind keeps them.
    v: [i32; 6],
    /// Operation 8's mode and points, operation 10's points.
    mode: i32,
    points: Option<Vec<[i32; 4]>>,
    /// Operation 34's octave strengths (`aShortArray9730`), when listed.
    shorts: Option<Vec<i16>>,
}

impl Settings {
    /// The defaults of each kind's fields.
    fn new(kind: u8) -> Self {
        let v = match kind {
            0 => [4096, 0, 0, 0, 0, 0],
            5 => [1, 1, 0, 0, 0, 0],
            6 => [0, 4096, 0, 0, 0, 0],
            7 => [6, 0, 0, 0, 0, 0],
            // `aBool9878` (mirror x), `aBool9875` (mirror y).
            9 => [1, 1, 0, 0, 0, 0],
            // `anInt9961`, `anInt9962` (cells), `anInt9959` (seed), `anInt9960` (jitter),
            // `anInt9963` (output), `anInt9949` (distance).
            15 => [5, 5, 0, 2048, 2, 1],
            // `anInt9836`.
            19 => [32768, 0, 0, 0, 0, 0],
            // `anInt9673`, `anInt9674`: tiles across and down.
            20 => [4, 4, 0, 0, 0, 0],
            // `anInt9759`, `anInt9760`.
            30 => [1024, 3072, 0, 0, 0, 0],
            // `anInt9810`, `anInt9813`, `anInt9807`.
            32 => [4096, 3216, 3216, 0, 0, 0],
            // `aBool9726`, `anInt9736` (octaves), `anInt9737` (persistence), `anInt9725`,
            // `anInt9735` (cells), `anInt9733` (seed).
            34 => [1, 4, 1638, 4, 4, 0],
            _ => [0; 6],
        };
        Settings { mono: MONO[kind as usize], v, mode: 0, points: None, shorts: None }
    }

    /// The settings of the ported kinds (`MaterialPropN.decode`); false for any other.
    fn read(&mut self, r: &mut Reader, kind: u8, setting: u8) -> Result<bool> {
        match (kind, setting) {
            // `(byte << 12) / 255`.
            (0, 0) => self.v[0] = ((r.u8()? as i32) << 12) / 255,
            (1, 0) => self.v[0] = r.u24()? as i32,
            (5, 0) | (5, 1) => self.v[setting as usize] = r.u8()? as i32,
            (6, 0) | (6, 1) => self.v[setting as usize] = r.u16()? as i32,
            (7, 0) => self.v[0] = r.u8()? as i32,
            (9, 0) | (9, 1) => self.v[setting as usize] = i32::from(r.u8()? == 1),
            (17, 0) => self.v[0] = r.i16()? as i32,
            // `(readByte() << 12) / 100`.
            (17, 1) | (17, 2) => self.v[setting as usize] = ((r.i8()? as i32) << 12) / 100,
            (19, 0) => self.v[0] = (r.u16()? as i32) << 4,
            (20, 0) | (20, 1) => self.v[setting as usize] = r.u8()? as i32,
            (30, 0) | (30, 1) | (32, 0..=2) => self.v[setting as usize] = r.u16()? as i32,
            (15, 0) => self.v[..2].fill(r.u8()? as i32),
            (15, 1) => self.v[2] = r.u8()? as i32,
            (15, 2) => self.v[3] = r.u16()? as i32,
            (15, 3) | (15, 4) => self.v[setting as usize + 1] = r.u8()? as i32,
            (15, 5) | (15, 6) => self.v[setting as usize - 5] = r.u8()? as i32,
            (34, 0) => self.v[0] = i32::from(r.u8()? == 1),
            (34, 1) => self.v[1] = r.u8()? as i32,
            (34, 2) => {
                // A negative persistence lists the current octave count's strengths.
                self.v[2] = r.i16()? as i32;
                if self.v[2] < 0 {
                    self.shorts = Some((0..self.v[1]).map(|_| r.i16()).collect::<Result<_>>()?);
                }
            }
            (34, 3) => self.v[3..5].fill(r.u8()? as i32),
            (34, 4) => self.v[5] = r.u8()? as i32,
            (34, 5) | (34, 6) => self.v[setting as usize - 2] = r.u8()? as i32,
            (5, 2) | (6, 2) | (7, 1) | (9, 2) | (19, 1) | (21, 0) | (22, 0) | (30, 2) => self.mono = r.u8()? == 1,
            (8, 0) => {
                self.mode = r.u8()? as i32;
                let n = r.u8()?;
                self.points = Some((0..n).map(|_| Ok([r.u16()? as i32, r.u16()? as i32, 0, 0])).collect::<Result<_>>()?);
            }
            (10, 0) => {
                let preset = r.u8()?;
                self.points = Some(if preset == 0 {
                    let n = r.u8()?;
                    (0..n)
                        .map(|_| Ok([r.u16()? as i32, (r.u8()? as i32) << 4, (r.u8()? as i32) << 4, (r.u8()? as i32) << 4]))
                        .collect::<Result<_>>()?
                } else {
                    ramp_preset(preset)?
                });
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// `method12321`: the kind's parameters made ready.
    fn finish(self, kind: u8) -> Result<(bool, Params)> {
        let p = match kind {
            0 => Params::Constant(self.v[0]),
            // `MaterialProp1.method15378`.
            1 => {
                let c = self.v[0];
                Params::Colour([(c & 0xff0000) >> 12, (c & 0xff00) >> 4, (c & 0xff) << 4])
            }
            2 => Params::Horizontal,
            3 => Params::Vertical,
            5 => Params::Blur(self.v[0], self.v[1]),
            6 => Params::Clamp(self.v[0], self.v[1]),
            7 => Params::Blend(self.v[0]),
            8 => Params::Curve(Box::new(curve(self.mode, self.points.unwrap_or_else(|| vec![[0; 4], [4096, 4096, 0, 0]]))?)),
            10 => Params::Ramp(Box::new(ramp(&self.points.map_or_else(|| ramp_preset(1), Ok)?)?)),
            9 => Params::Mirror(self.v[0] == 1, self.v[1] == 1),
            15 => {
                let [cx, cy, seed, jitter, output, distance] = self.v;
                // `method15412`: its own generator of the same seed; no jitter leaves them 0.
                let mut offsets = Box::new([0i16; 512]);
                if jitter > 0 {
                    let mut random = JavaRandom::new(seed as i64);
                    offsets.iter_mut().for_each(|o| *o = random.below(jitter) as i16);
                }
                Params::Cells(Box::new(Cells { cells: (cx, cy), output, distance, perm: permutation(seed), offsets }))
            }
            17 => Params::Hsl([self.v[0], self.v[1], self.v[2]]),
            18 => Params::TiledSprite,
            36 => Params::Texture,
            39 => Params::Sprite,
            34 => noise(self.v, self.shorts),
            13 => Params::Hash,
            19 => Params::Warp(self.v[0]),
            20 => Params::Tile(self.v[0], self.v[1]),
            21 => Params::Mix,
            22 => Params::Invert,
            // `method12321`: the span from low to high.
            30 => Params::Range(self.v[0], self.v[1] - self.v[0]),
            32 => Params::Emboss(self.v[0], emboss_light(self.v[1], self.v[2])),
            _ => Params::Unported,
        };
        Ok((self.mono, p))
    }
}

/// `MaterialProp34.method12321`: the octaves' strengths and frequencies (`method15199`), the
/// quiet last octaves dropped. Settings the reference can't draw (no strengths) stay unported.
fn noise(v: [i32; 6], listed: Option<Vec<i16>>) -> Params {
    let [centred, mut octaves, persistence, cx, cy, seed] = v;
    let frequencies = |n: i32| (0..n).map(|i| 2f64.powi(i) as i32 as i16).collect::<Vec<_>>();
    let (mut amplitude, mut frequency) = if persistence > 0 {
        let p = (persistence as f32 / 4096.0) as f64;
        ((0..octaves).map(|i| (p.powf(i as f64) * 4096.0) as i32 as i16).collect(), frequencies(octaves))
    } else {
        match listed {
            Some(a) if a.len() == octaves as usize => (a, frequencies(octaves)),
            _ => return Params::Unported,
        }
    };
    while octaves > 1 && (-8..=8).contains(&amplitude[octaves as usize - 1]) {
        octaves -= 1;
    }
    if octaves < 1 {
        return Params::Unported;
    }
    amplitude.truncate(octaves as usize);
    frequency.truncate(octaves as usize);
    Params::Noise(Box::new(Noise { centred: centred == 1, cells: (cx, cy), amplitude, frequency, perm: permutation(seed) }))
}

/// Java's `int /`, failing where Java throws.
fn div(a: i32, b: i32) -> Result<i32> {
    if b == 0 {
        return corrupt("material divides by zero".to_string());
    }
    Ok(a.wrapping_div(b))
}

/// `Class316.anIntArray3675`: cos(2π i / 255) × 4096.
pub fn cos_table() -> [i32; 256] {
    std::array::from_fn(|i| ((std::f64::consts::TAU * (i as f64 / 255.0)).cos() * 4096.0) as i32)
}

/// `Class316.anIntArray3677`: sin(2π i / 255) × 4096.
pub fn sin_table() -> [i32; 256] {
    std::array::from_fn(|i| ((std::f64::consts::TAU * (i as f64 / 255.0)).sin() * 4096.0) as i32)
}

/// `MaterialProp32.method15266`: the light direction from its two angles (× 4096 radians),
/// made length 4096.
fn emboss_light(yaw: i32, pitch: i32) -> [i32; 3] {
    let d = ((pitch as f32 / 4096.0) as f64).cos();
    let mut l = [
        (((yaw as f32 / 4096.0) as f64).sin() * d * 4096.0) as i32,
        (((yaw as f32 / 4096.0) as f64).cos() * d * 4096.0) as i32,
        (((pitch as f32 / 4096.0) as f64).sin() * 4096.0) as i32,
    ];
    let sum = l.iter().map(|&c| c.wrapping_mul(c) >> 12).fold(0i32, |a, c| a.wrapping_add(c));
    let len = (((sum >> 12) as f64).sqrt() * 4096.0) as i32;
    if len != 0 {
        l = l.map(|c| c.wrapping_shl(12).wrapping_div(len));
    }
    l
}

/// `MaterialProp8.method15258`: the curve through `points` (x, y), mode 1 cosine, 2 cubic
/// (with the ends extended, `method15259`), else linear.
fn curve(mode: i32, points: Vec<[i32; 4]>) -> Result<[i16; 257]> {
    if points.len() < 2 {
        return corrupt("material curve with fewer than 2 points".to_string());
    }
    let n = points.len();
    let (first, last) = if mode == 2 {
        let (a, b, c, d) = (points[0], points[1], points[n - 2], points[n - 1]);
        ([a[0] - b[0] + a[0], a[1] - b[1] + a[1]], [c[0] - d[0] + c[0], c[1] - d[1] + c[1]])
    } else {
        ([0, 0], [0, 0])
    };
    let y_of = |i: i64| -> i32 {
        if i < 0 {
            first[1]
        } else if i as usize >= n {
            last[1]
        } else {
            points[i as usize][1]
        }
    };
    let cos = cos_table();
    let mut out = [0i16; 257];
    for (i, o) in out.iter_mut().enumerate() {
        let x = (i as i32) << 4;
        let mut k = 1;
        while k < n - 1 && points[k][0] <= x {
            k += 1;
        }
        let (a, b) = (points[k - 1], points[k]);
        let t = div((x - a[0]).wrapping_shl(12), b[0] - a[0])?;
        let v = match mode {
            1 => {
                let f = (4096 - cos[((t >> 5) & 0xff) as usize]) >> 1;
                (f.wrapping_mul(b[1]) + (4096 - f).wrapping_mul(a[1])) >> 12
            }
            2 => {
                let (p0, p1, p2, p3) = (y_of(k as i64 - 2), a[1], b[1], y_of(k as i64 + 1));
                let t2 = t.wrapping_mul(t) >> 12;
                let c3 = p1 + (p3 - p2 - p0);
                let c2 = p0 - p1 - c3;
                let c1 = p2 - p0;
                (t2.wrapping_mul(c3.wrapping_mul(t) >> 12) >> 12) + (c2.wrapping_mul(t2) >> 12) + (t.wrapping_mul(c1) >> 12) + p1
            }
            _ => (t.wrapping_mul(b[1]) + (4096 - t).wrapping_mul(a[1])) >> 12,
        };
        *o = v.clamp(-32767, 32767) as i16;
    }
    Ok(out)
}

/// `MaterialProp10.method15268`: the colour ramp through `points` (x, r, g, b), as RGB.
fn ramp(points: &[[i32; 4]]) -> Result<[i32; 257]> {
    let mut out = [0i32; 257];
    let n = points.len();
    if n == 0 {
        return Ok(out);
    }
    for (i, o) in out.iter_mut().enumerate() {
        let x = (i as i32) << 4;
        let k = points.iter().take_while(|p| x >= p[0]).count();
        let c = if k < n && k > 0 {
            let (a, b) = (points[k - 1], points[k]);
            let t = div((x - a[0]).wrapping_shl(12), b[0] - a[0])?;
            let u = 4096 - t;
            [1, 2, 3].map(|j| (t.wrapping_mul(b[j]) + u.wrapping_mul(a[j])) >> 12)
        } else {
            let p = points[if k < n { k } else { n - 1 }];
            [p[1], p[2], p[3]]
        };
        let [r, g, b] = c.map(|v| (v >> 4).clamp(0, 255));
        *o = r << 16 | g << 8 | b;
    }
    Ok(out)
}

/// `MaterialProp10.method15269`: the built-in ramps 1..=6.
fn ramp_preset(preset: u8) -> Result<Vec<[i32; 4]>> {
    Ok(match preset {
        1 => vec![[0, 0, 0, 0], [4096, 4096, 4096, 4096]],
        2 => vec![
            [0, 2650, 2602, 2361],
            [2867, 2313, 1799, 1558],
            [3072, 2618, 1734, 1413],
            [3276, 2296, 1220, 947],
            [3481, 2072, 963, 722],
            [3686, 2730, 2152, 1766],
            [3891, 2232, 1060, 915],
            [4096, 1686, 1413, 1140],
        ],
        3 => vec![
            [0, 0, 0, 4096],
            [663, 0, 4096, 4096],
            [1363, 0, 4096, 0],
            [2048, 4096, 4096, 0],
            [2727, 4096, 0, 0],
            [3411, 4096, 0, 4096],
            [4096, 0, 0, 4096],
        ],
        4 => vec![
            [0, 0, 0, 0],
            [1843, 0, 0, 1493],
            [2457, 0, 0, 2939],
            [2781, 0, 1124, 3565],
            [3481, 546, 3084, 4031],
            [4096, 4096, 4096, 4096],
        ],
        5 => vec![
            [0, 80, 192, 321],
            [155, 321, 449, 562],
            [389, 578, 690, 803],
            [671, 947, 995, 1140],
            [897, 1285, 1397, 1509],
            [1175, 1525, 1429, 1413],
            [1368, 1734, 1461, 1333],
            [1507, 1413, 1525, 1702],
            [1736, 1108, 1590, 2056],
            [2088, 1766, 2056, 2666],
            [2355, 2409, 2586, 3276],
            [2691, 3116, 3148, 3228],
            [3031, 3806, 3710, 3196],
            [3522, 3437, 3421, 3019],
            [3727, 3116, 3148, 3228],
            [4096, 2377, 2505, 2746],
        ],
        6 => vec![[2048, 0, 4096, 0], [2867, 4096, 4096, 0], [3276, 4096, 4096, 0], [4096, 4096, 0, 0]],
        p => return corrupt(format!("material ramp preset {p}")),
    })
}

/// Reads a setting of an operation not ported yet (each `MaterialPropN.decode`); a sprite or
/// texture id comes back.
fn skip_setting(r: &mut Reader, kind: u8, setting: u8) -> Result<Option<i32>> {
    let (bytes, shorts): (usize, usize) = match (kind, setting) {
        (4, 0..=1) | (12, 0) | (12, 1) | (12, 3) | (16, 0..=1) | (23, 0) | (27, 0) | (27, 2) | (28, 0) | (28, 6) => (1, 0),
        (29, 1) | (33, 1) | (38, 0) | (38, 2) => (1, 0),
        (4, 2..=7) | (11, 0..=2) | (14, 0) | (16, 2) | (25, 0..=3) | (26, 0..=1) | (27, 1) | (28, 1..=5) | (28, 7..=8) => (0, 1),
        (31, 0..=3) | (33, 0) | (35, 0) | (37, 0..=6) | (38, 1) | (38, 3..=4) => (0, 1),
        (25, 4) => (3, 0),
        (29, 0) => {
            let n = r.u8()?;
            for _ in 0..n {
                let size = match r.u8()? {
                    0 => 12,
                    1 => 20,
                    2 | 3 => 15,
                    _ => 0,
                };
                r.bytes(size)?;
            }
            return Ok(None);
        }
        (36, 0) | (39, 0) | (18, 0) => return Ok(Some(r.u16()? as i32)),
        // Any other setting reads nothing.
        _ => (0, 0),
    };
    r.bytes(bytes + shorts * 2)?;
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materials_wire_their_operations() {
        // Two operations: 0 (a constant, setting 0 = 128) and 5 (setting 2 = 1, input 0);
        // outputs 1, 1, 0.
        let data = [2, 0, 0, 255, 1, 0, 128, 0, 5, 255, 1, 2, 1, 0, 1, 1, 0];
        let m = Material::decode(&data).unwrap();
        assert_eq!(m.ops.iter().map(|o| (o.kind, o.inputs.clone())).collect::<Vec<_>>(), vec![(0, vec![]), (5, vec![0])]);
        assert_eq!(m.outputs, [1, 1, 0]);
        // A sprite operation keeps its sprite id.
        let sprite = Material::decode(&[1, 0, 39, 255, 1, 0, 0, 7, 0, 0, 0]).unwrap();
        assert_eq!(sprite.ops[0].image, 7);
    }

    #[test]
    fn the_texture_table_reads_field_by_field() {
        // Two slots, the second defined: ground mesh (0 → true), half size, skip (1), brightness
        // 5, shadow −1, effect, effect param, colour 0x1234, speeds, aBool, brick (1), mipmaps,
        // repeat S/T, hdr, combine, effectParam2 (int), blend 2.
        let mut data = vec![0, 2, 0, 1, 0, 0, 1, 5, 0xff, 0, 0, 0x12, 0x34, 0, 0, 0, 1, 0, 0, 0, 0, 0];
        data.extend([0, 0, 0, 0, 2]);
        let t = texture_table(&data).unwrap();
        assert_eq!(t[0], None);
        let d = t[1].clone().unwrap();
        assert_eq!(
            (d.ground_mesh, d.skip_triangles, d.brightness, d.shadow, d.colour, d.brick, d.blend),
            (true, true, 5, -1, 0x1234, true, 2)
        );
    }

    #[test]
    fn the_noise_generator_matches_java() {
        // `new Random(0).nextInt()` and `new Random(42).nextInt()` in Java.
        assert_eq!(JavaRandom::new(0).next_int(), -1155484576);
        assert_eq!(JavaRandom::new(42).next_int(), -1170105035);
        // A shuffle keeps every byte once (0 twice: entry 255 starts 0) and copies it above 256.
        let p = permutation(3);
        let mut seen = p[..256].to_vec();
        seen.sort_unstable();
        assert_eq!(seen, (0..=255).map(|i: u32| if i == 0 { 0 } else { (i - 1) as u8 }).collect::<Vec<_>>());
        assert!((1..256).all(|i| p[i] == p[256 + i]));
    }

    #[test]
    fn noise_drops_its_quiet_octaves() {
        // Persistence 100: strengths 4096, 100, 2 → the last is dropped.
        let Params::Noise(n) = noise([1, 3, 100, 4, 4, 0], None) else { panic!("noise") };
        assert_eq!((n.amplitude, n.frequency), (vec![4096, 100], vec![1, 2]));
    }
}
