//! The client's own Vorbis variant for instrument samples (index 14). Port of `MIDIInstrument`
//! (setup in file 0/0 `method12261`, a sample's header `method12265`, packet decoding with
//! its inverse MDCT `method12267`, the decode loop `method12273`), `Class255` (codebook),
//! `Class338` (floor 1), `Class265` (residue) and `Class259` (mapping).
//!
//! Arithmetic is kept in `f32` where the reference uses `float`, `f64` where it uses `double`.

use super::error::{corrupt, Result};

/// `method12262` / `method12263` / `method12264`: least-significant-bit-first reader.
struct Bits<'a> {
    data: &'a [u8],
    byte: usize,
    bit: u32,
}

impl<'a> Bits<'a> {
    fn new(data: &'a [u8]) -> Self {
        Bits { data, byte: 0, bit: 0 }
    }

    fn cur(&self) -> i32 {
        // Past the end the reference would throw; reading zeros only matters for broken data.
        self.data.get(self.byte).copied().unwrap_or(0) as i32
    }

    fn bit(&mut self) -> i32 {
        let v = self.cur() >> self.bit & 1;
        self.bit += 1;
        self.byte += (self.bit >> 3) as usize;
        self.bit &= 7;
        v
    }

    fn bits(&mut self, mut n: u32) -> i32 {
        let (mut v, mut shift) = (0i32, 0u32);
        while n >= 8 - self.bit {
            let take = 8 - self.bit;
            let mask = (1i32 << take) - 1;
            v = v.wrapping_add((self.cur() >> self.bit & mask).wrapping_shl(shift));
            self.bit = 0;
            self.byte += 1;
            shift += take;
            n -= take;
        }
        if n > 0 {
            let mask = (1i32 << n) - 1;
            v = v.wrapping_add((self.cur() >> self.bit & mask).wrapping_shl(shift));
            self.bit += n;
        }
        v
    }
}

/// `Class159.method2739`: how many bits `v` needs (`ilog`).
fn ilog(v: i32) -> u32 {
    32 - (v as u32).leading_zeros()
}

/// `MIDIInstrument.method12300`: Vorbis' packed float.
fn unpack_float(v: i32) -> f32 {
    let mantissa = v & 0x1f_ffff;
    let exponent = (v & 0x7fe0_0000) >> 21;
    let m = if v < 0 { -mantissa } else { mantissa };
    (m as f64 * 2f64.powi(exponent - 788)) as f32
}

/// `Class277.method4906`: integer power.
fn pow_i(mut base: i32, mut exp: i32) -> i32 {
    let mut acc = 1i32;
    while exp > 1 {
        if exp & 1 != 0 {
            acc = base.wrapping_mul(acc);
        }
        base = base.wrapping_mul(base);
        exp >>= 1;
    }
    if exp == 1 {
        base.wrapping_mul(acc)
    } else {
        acc
    }
}

/// `Class255`.
struct Codebook {
    dimensions: usize,
    /// `anIntArray3151`: the Huffman tree (leaves stored as `!entry`).
    tree: Vec<i32>,
    /// `aFloatArrayArray3150`: vector values, when the codebook has a lookup.
    vectors: Option<Vec<Vec<f32>>>,
}

impl Codebook {
    fn read(b: &mut Bits) -> Self {
        b.bits(24);
        let dimensions = b.bits(16) as usize;
        let entries = b.bits(24) as usize;
        let mut lengths = vec![0i32; entries];
        if b.bit() != 0 {
            let (mut i, mut len) = (0usize, b.bits(5) + 1);
            while i < entries {
                let n = b.bits(ilog((entries - i) as i32));
                for _ in 0..n {
                    if i < entries {
                        lengths[i] = len;
                    }
                    i += 1;
                }
                len += 1;
            }
        } else {
            let sparse = b.bit() != 0;
            for l in &mut lengths {
                *l = if sparse && b.bit() == 0 { 0 } else { b.bits(5) + 1 };
            }
        }
        let tree = build_tree(&lengths);
        let lookup = b.bits(4);
        let vectors = (lookup > 0).then(|| {
            let min = unpack_float(b.bits(32));
            let delta = unpack_float(b.bits(32));
            let value_bits = (b.bits(4) + 1) as u32;
            let sequence = b.bit() != 0;
            let count = if lookup == 1 { lookup1_values(entries as i32, dimensions as i32) } else { (entries * dimensions) as i32 };
            let multiplicands: Vec<i32> = (0..count).map(|_| b.bits(value_bits)).collect();
            let mut vectors = vec![vec![0f32; dimensions]; entries];
            for (e, vector) in vectors.iter_mut().enumerate() {
                let mut last = 0f32;
                let (mut divisor, mut offset) = (1i32, e * dimensions);
                for v in vector.iter_mut() {
                    let m = if lookup == 1 {
                        multiplicands[((e as i32 / divisor) % count) as usize]
                    } else {
                        let m = multiplicands[offset];
                        offset += 1;
                        m
                    };
                    let value = m as f32 * delta + min + last;
                    *v = value;
                    if sequence {
                        last = value;
                    }
                    divisor = divisor.wrapping_mul(count);
                }
            }
            vectors
        });
        Codebook { dimensions, tree, vectors }
    }

    /// `method4384`: one entry.
    fn entry(&self, b: &mut Bits) -> usize {
        let mut i = 0usize;
        while self.tree[i] >= 0 {
            i = if b.bit() != 0 { self.tree[i] as usize } else { i + 1 };
        }
        !self.tree[i] as usize
    }

    /// `method4382`: one vector.
    fn vector(&self, b: &mut Bits) -> &[f32] {
        let e = self.entry(b);
        &self.vectors.as_ref().expect("residue codebook without lookup")[e]
    }
}

/// `method4397`.
fn lookup1_values(entries: i32, dimensions: i32) -> i32 {
    let mut v = (entries as f64).powf(1.0 / dimensions as f64) as i32 + 1;
    while pow_i(v, dimensions) > entries {
        v -= 1;
    }
    v
}

/// `method4383`: canonical Huffman codes into the reference's array tree.
fn build_tree(lengths: &[i32]) -> Vec<i32> {
    let mut codes = vec![0i32; lengths.len()];
    let mut next = [0i32; 33];
    for (i, &len) in lengths.iter().enumerate() {
        if len == 0 {
            continue;
        }
        let len = len as usize;
        let bit = 1i32.wrapping_shl(32 - len as u32);
        let code = next[len];
        codes[i] = code;
        let replacement;
        if code & bit != 0 {
            replacement = next[len - 1];
        } else {
            replacement = code | bit;
            for shorter in (1..len).rev() {
                let c = next[shorter];
                if c != code {
                    break;
                }
                let b = 1i32.wrapping_shl(32 - shorter as u32);
                if c & b != 0 {
                    next[shorter] = next[shorter - 1];
                    break;
                }
                next[shorter] = c | b;
            }
        }
        next[len] = replacement;
        for slot in next.iter_mut().skip(len + 1) {
            if *slot == code {
                *slot = replacement;
            }
        }
    }
    let mut tree = vec![0i32; 8];
    let mut used = 0usize;
    for (i, &len) in lengths.iter().enumerate() {
        if len == 0 {
            continue;
        }
        let code = codes[i];
        let mut node = 0usize;
        for k in 0..len {
            let mask = (i32::MIN as u32 >> k) as i32;
            if code & mask != 0 {
                if tree[node] == 0 {
                    tree[node] = used as i32;
                }
                node = tree[node] as usize;
            } else {
                node += 1;
            }
            if node >= tree.len() {
                tree.resize(tree.len() * 2, 0);
            }
        }
        tree[node] = !(i as i32);
        if node >= used {
            used = node + 1;
        }
    }
    tree
}

/// `Class338.anIntArray3973`.
const FLOOR_RANGES: [i32; 4] = [256, 128, 86, 64];

/// `Class338.aFloatArray3971`: Vorbis' floor 1 inverse dB table.
#[rustfmt::skip]
static INVERSE_DB: [f32; 256] = [
    1.0649863E-7, 1.1341951E-7, 1.2079015E-7, 1.2863978E-7, 1.369995E-7, 1.459025E-7, 1.5538409E-7, 1.6548181E-7,
    1.7623574E-7, 1.8768856E-7, 1.998856E-7, 2.128753E-7, 2.2670913E-7, 2.4144197E-7, 2.5713223E-7, 2.7384212E-7,
    2.9163792E-7, 3.1059022E-7, 3.307741E-7, 3.5226967E-7, 3.7516213E-7, 3.995423E-7, 4.255068E-7, 4.5315863E-7,
    4.8260745E-7, 5.1397E-7, 5.4737063E-7, 5.829419E-7, 6.208247E-7, 6.611694E-7, 7.041359E-7, 7.4989464E-7,
    7.98627E-7, 8.505263E-7, 9.057983E-7, 9.646621E-7, 1.0273513E-6, 1.0941144E-6, 1.1652161E-6, 1.2409384E-6,
    1.3215816E-6, 1.4074654E-6, 1.4989305E-6, 1.5963394E-6, 1.7000785E-6, 1.8105592E-6, 1.9282195E-6, 2.053526E-6,
    2.1869757E-6, 2.3290977E-6, 2.4804558E-6, 2.6416496E-6, 2.813319E-6, 2.9961443E-6, 3.1908505E-6, 3.39821E-6,
    3.619045E-6, 3.8542307E-6, 4.1047006E-6, 4.371447E-6, 4.6555283E-6, 4.958071E-6, 5.280274E-6, 5.623416E-6,
    5.988857E-6, 6.3780467E-6, 6.7925284E-6, 7.2339453E-6, 7.704048E-6, 8.2047E-6, 8.737888E-6, 9.305725E-6,
    9.910464E-6, 1.0554501E-5, 1.1240392E-5, 1.1970856E-5, 1.2748789E-5, 1.3577278E-5, 1.4459606E-5, 1.5399271E-5,
    1.6400005E-5, 1.7465769E-5, 1.8600793E-5, 1.9809577E-5, 2.1096914E-5, 2.2467912E-5, 2.3928002E-5, 2.5482977E-5,
    2.7139005E-5, 2.890265E-5, 3.078091E-5, 3.2781227E-5, 3.4911533E-5, 3.718028E-5, 3.9596467E-5, 4.2169668E-5,
    4.491009E-5, 4.7828602E-5, 5.0936775E-5, 5.424693E-5, 5.7772202E-5, 6.152657E-5, 6.552491E-5, 6.9783084E-5,
    7.4317984E-5, 7.914758E-5, 8.429104E-5, 8.976875E-5, 9.560242E-5, 1.0181521E-4, 1.0843174E-4, 1.1547824E-4,
    1.2298267E-4, 1.3097477E-4, 1.3948625E-4, 1.4855085E-4, 1.5820454E-4, 1.6848555E-4, 1.7943469E-4, 1.9109536E-4,
    2.0351382E-4, 2.167393E-4, 2.3082423E-4, 2.4582449E-4, 2.6179955E-4, 2.7881275E-4, 2.9693157E-4, 3.1622787E-4,
    3.3677815E-4, 3.5866388E-4, 3.8197188E-4, 4.0679457E-4, 4.3323037E-4, 4.613841E-4, 4.913675E-4, 5.2329927E-4,
    5.573062E-4, 5.935231E-4, 6.320936E-4, 6.731706E-4, 7.16917E-4, 7.635063E-4, 8.1312325E-4, 8.6596457E-4,
    9.2223985E-4, 9.821722E-4, 0.0010459992, 0.0011139743, 0.0011863665, 0.0012634633, 0.0013455702, 0.0014330129,
    0.0015261382, 0.0016253153, 0.0017309374, 0.0018434235, 0.0019632196, 0.0020908006, 0.0022266726, 0.0023713743,
    0.0025254795, 0.0026895993, 0.0028643848, 0.0030505287, 0.003248769, 0.0034598925, 0.0036847359, 0.0039241905,
    0.0041792067, 0.004450795, 0.004740033, 0.005048067, 0.0053761187, 0.005725489, 0.0060975635, 0.0064938175,
    0.0069158226, 0.0073652514, 0.007843887, 0.008353627, 0.008896492, 0.009474637, 0.010090352, 0.01074608,
    0.011444421, 0.012188144, 0.012980198, 0.013823725, 0.014722068, 0.015678791, 0.016697686, 0.017782796,
    0.018938422, 0.020169148, 0.021479854, 0.022875736, 0.02436233, 0.025945531, 0.027631618, 0.029427277,
    0.031339627, 0.03337625, 0.035545226, 0.037855156, 0.0403152, 0.042935107, 0.045725275, 0.048696756,
    0.05186135, 0.05523159, 0.05882085, 0.062643364, 0.06671428, 0.07104975, 0.075666964, 0.08058423,
    0.08582105, 0.09139818, 0.097337745, 0.1036633, 0.11039993, 0.11757434, 0.12521498, 0.13335215,
    0.14201812, 0.15124726, 0.16107617, 0.1715438, 0.18269168, 0.19456401, 0.20720787, 0.22067343,
    0.23501402, 0.25028655, 0.26655158, 0.28387362, 0.3023213, 0.32196787, 0.34289113, 0.36517414,
    0.3889052, 0.41417846, 0.44109413, 0.4697589, 0.50028646, 0.53279793, 0.5674221, 0.6042964,
    0.64356697, 0.6853896, 0.72993004, 0.777365, 0.8278826, 0.88168305, 0.9389798, 1.0,
];

/// `Class338`: floor type 1.
struct Floor {
    partition_class: Vec<usize>,
    class_dimensions: Vec<usize>,
    class_subclasses: Vec<u32>,
    class_masterbook: Vec<usize>,
    subclass_books: Vec<Vec<i32>>,
    multiplier: i32,
    x_list: Vec<i32>,
}

impl Floor {
    fn read(b: &mut Bits) -> Result<Self> {
        if b.bits(16) != 1 {
            return corrupt("vorbis floor type isn't 1");
        }
        let partitions = b.bits(5) as usize;
        let partition_class: Vec<usize> = (0..partitions).map(|_| b.bits(4) as usize).collect();
        let classes = partition_class.iter().map(|&c| c + 1).max().unwrap_or(0);
        let (mut class_dimensions, mut class_subclasses, mut class_masterbook, mut subclass_books) =
            (vec![0; classes], vec![0; classes], vec![0; classes], vec![Vec::new(); classes]);
        for c in 0..classes {
            class_dimensions[c] = (b.bits(3) + 1) as usize;
            class_subclasses[c] = b.bits(2) as u32;
            if class_subclasses[c] != 0 {
                class_masterbook[c] = b.bits(8) as usize;
            }
            subclass_books[c] = (0..1 << class_subclasses[c]).map(|_| b.bits(8) - 1).collect();
        }
        let multiplier = b.bits(2) + 1;
        let range_bits = b.bits(4) as u32;
        let mut x_list = vec![0, 1 << range_bits];
        for &c in &partition_class {
            for _ in 0..class_dimensions[c] {
                x_list.push(b.bits(range_bits));
            }
        }
        Ok(Floor { partition_class, class_dimensions, class_subclasses, class_masterbook, subclass_books, multiplier, x_list })
    }

    /// `method6024`: the packet's Y values, or `None` when the floor is unused.
    fn decode(&self, b: &mut Bits, books: &[Codebook]) -> Option<Vec<i32>> {
        if b.bit() == 0 {
            return None;
        }
        let range = FLOOR_RANGES[(self.multiplier - 1) as usize];
        let bits = ilog(range - 1);
        let mut y = vec![0i32; self.x_list.len()];
        y[0] = b.bits(bits);
        y[1] = b.bits(bits);
        let mut at = 2;
        for &c in &self.partition_class {
            let sub = self.class_subclasses[c];
            let mask = (1i32 << sub) - 1;
            let mut cval = if sub > 0 { books[self.class_masterbook[c]].entry(b) as i32 } else { 0 };
            for _ in 0..self.class_dimensions[c] {
                let book = self.subclass_books[c][(cval & mask) as usize];
                cval = ((cval as u32) >> sub) as i32;
                y[at] = if book >= 0 { books[book as usize].entry(b) as i32 } else { 0 };
                at += 1;
            }
        }
        Some(y)
    }

    /// `method6025`: multiplies the first `n` spectrum values by the floor curve.
    fn apply(&self, y_coded: Vec<i32>, out: &mut [f32], n: usize) {
        let count = self.x_list.len();
        let range = FLOOR_RANGES[(self.multiplier - 1) as usize];
        let mut xs = self.x_list.clone();
        let mut ys = y_coded;
        let mut used = vec![false; count];
        used[0] = true;
        used[1] = true;
        for i in 2..count {
            let low = neighbour(&xs, i, |v, x, best| v < x && v > best, i32::MIN);
            let high = neighbour(&xs, i, |v, x, best| v > x && v < best, i32::MAX);
            let predicted = render_point(xs[low], ys[low], xs[high], ys[high], xs[i]);
            let value = ys[i];
            let high_room = range - predicted;
            let room = high_room.min(predicted) << 1;
            if value != 0 {
                used[low] = true;
                used[high] = true;
                used[i] = true;
                ys[i] = if value >= room {
                    if high_room > predicted {
                        predicted + (value - predicted)
                    } else {
                        high_room + (predicted - value) - 1
                    }
                } else if value & 1 != 0 {
                    predicted - (value + 1) / 2
                } else {
                    value / 2 + predicted
                };
            } else {
                used[i] = false;
                ys[i] = predicted;
            }
        }
        sort(&mut xs, &mut ys, &mut used, 0, count as isize - 1);
        let (mut lx, mut ly) = (0i32, ys[0] * self.multiplier);
        for i in 1..count {
            if used[i] {
                let (hx, hy) = (xs[i], ys[i] * self.multiplier);
                render_line(lx, ly, hx, hy, out, n as i32);
                if hx >= n as i32 {
                    return;
                }
                lx = hx;
                ly = hy;
            }
        }
        let level = INVERSE_DB[ly as usize];
        for v in out.iter_mut().take(n).skip(lx as usize) {
            *v *= level;
        }
    }
}

/// `method6022` / `method6020`: the nearest earlier X below (or above) `xs[i]`.
fn neighbour(xs: &[i32], i: usize, better: impl Fn(i32, i32, i32) -> bool, start: i32) -> usize {
    let (x, mut best, mut found) = (xs[i], start, usize::MAX);
    for (k, &v) in xs.iter().enumerate().take(i) {
        if better(v, x, best) {
            found = k;
            best = v;
        }
    }
    found
}

/// `method6019`.
fn render_point(x0: i32, y0: i32, x1: i32, y1: i32, x: i32) -> i32 {
    let dy = y1 - y0;
    let off = (x - x0) * dy.abs() / (x1 - x0);
    if dy < 0 {
        y0 - off
    } else {
        y0 + off
    }
}

/// `method6021`.
fn render_line(x0: i32, y0: i32, mut x1: i32, y1: i32, out: &mut [f32], n: i32) {
    let dy = y1 - y0;
    let adx = x1 - x0;
    let base = dy / adx;
    let step = if dy < 0 { base - 1 } else { base + 1 };
    let ady = dy.abs() - base.abs() * adx;
    let (mut y, mut err) = (y0, 0);
    out[x0 as usize] *= INVERSE_DB[y0 as usize];
    if x1 > n {
        x1 = n;
    }
    for x in x0 + 1..x1 {
        err += ady;
        if err >= adx {
            err -= adx;
            y += step;
        } else {
            y += base;
        }
        out[x as usize] *= INVERSE_DB[y as usize];
    }
}

/// `method6035`: the reference's quicksort of the floor points by X.
fn sort(xs: &mut [i32], ys: &mut [i32], used: &mut [bool], lo: isize, hi: isize) {
    if lo >= hi {
        return;
    }
    let (l, h) = (lo as usize, hi as usize);
    let (px, py, pu) = (xs[l], ys[l], used[l]);
    let mut at = l;
    for k in l + 1..=h {
        if xs[k] < px {
            xs[at] = xs[k];
            ys[at] = ys[k];
            used[at] = used[k];
            at += 1;
            xs[k] = xs[at];
            ys[k] = ys[at];
            used[k] = used[at];
        }
    }
    xs[at] = px;
    ys[at] = py;
    used[at] = pu;
    sort(xs, ys, used, lo, at as isize - 1);
    sort(xs, ys, used, at as isize + 1, hi);
}

/// `Class265`.
struct Residue {
    kind: i32,
    begin: i32,
    end: i32,
    partition_size: i32,
    classifications: i32,
    classbook: usize,
    books: Vec<i32>,
}

impl Residue {
    fn read(b: &mut Bits) -> Self {
        let kind = b.bits(16);
        let begin = b.bits(24);
        let end = b.bits(24);
        let partition_size = b.bits(24) + 1;
        let classifications = b.bits(6) + 1;
        let classbook = b.bits(8) as usize;
        let cascade: Vec<i32> = (0..classifications)
            .map(|_| {
                let low = b.bits(3);
                let high = if b.bit() != 0 { b.bits(5) } else { 0 };
                high << 3 | low
            })
            .collect();
        let books = (0..classifications * 8).map(|i| if cascade[(i >> 3) as usize] & 1 << (i & 7) != 0 { b.bits(8) } else { -1 }).collect();
        Residue { kind, begin, end, partition_size, classifications, classbook, books }
    }

    /// `method4790`.
    fn decode(&self, b: &mut Bits, books: &[Codebook], out: &mut [f32], n: usize, skip: bool) {
        out[..n].fill(0.0);
        if skip {
            return;
        }
        let per_word = books[self.classbook].dimensions;
        let partitions = ((self.end - self.begin) / self.partition_size) as usize;
        let mut classes = vec![0i32; partitions];
        for pass in 0..8 {
            let mut p = 0usize;
            while p < partitions {
                if pass == 0 {
                    let mut word = books[self.classbook].entry(b) as i32;
                    for k in (0..per_word).rev() {
                        if p + k < partitions {
                            classes[p + k] = word % self.classifications;
                        }
                        word /= self.classifications;
                    }
                }
                for _ in 0..per_word {
                    let book = self.books[pass + classes[p] as usize * 8];
                    if book >= 0 {
                        let start = p as i32 * self.partition_size + self.begin;
                        let cb = &books[book as usize];
                        if self.kind == 0 {
                            let step = self.partition_size / cb.dimensions as i32;
                            for k in 0..step {
                                let v = cb.vector(b).to_vec();
                                for (d, value) in v.iter().enumerate() {
                                    out[(start + k + d as i32 * step) as usize] += value;
                                }
                            }
                        } else {
                            let mut k = 0;
                            while k < self.partition_size {
                                let v = cb.vector(b).to_vec();
                                for value in v {
                                    out[(start + k) as usize] += value;
                                    k += 1;
                                }
                            }
                        }
                    }
                    p += 1;
                    if p >= partitions {
                        break;
                    }
                }
            }
        }
    }
}

/// `Class259`.
struct Mapping {
    floor_submap: usize,
    floors: Vec<usize>,
    residues: Vec<usize>,
}

impl Mapping {
    fn read(b: &mut Bits) -> Self {
        b.bits(16);
        let submaps = if b.bit() != 0 { b.bits(4) + 1 } else { 1 } as usize;
        if b.bit() != 0 {
            b.bits(8);
        }
        b.bits(2);
        let floor_submap = if submaps > 1 { b.bits(4) as usize } else { 0 };
        let (mut floors, mut residues) = (vec![0; submaps], vec![0; submaps]);
        for i in 0..submaps {
            b.bits(8);
            floors[i] = b.bits(8) as usize;
            residues[i] = b.bits(8) as usize;
        }
        Mapping { floor_submap, floors, residues }
    }
}

/// The trig tables and bit reversal of one block size (`method12261`'s loop).
struct BlockTables {
    a: Vec<f32>,
    b: Vec<f32>,
    c: Vec<f32>,
    reversed: Vec<usize>,
}

impl BlockTables {
    fn new(n: usize) -> Self {
        let (half, quarter, eighth) = (n >> 1, n >> 2, n >> 3);
        let pi = std::f64::consts::PI;
        let mut a = vec![0f32; half];
        for i in 0..quarter {
            a[i * 2] = ((i * 4) as f64 * pi / n as f64).cos() as f32;
            a[i * 2 + 1] = -(((i * 4) as f64 * pi / n as f64).sin() as f32);
        }
        let mut b = vec![0f32; half];
        for i in 0..quarter {
            b[i * 2] = ((i * 2 + 1) as f64 * pi / (n * 2) as f64).cos() as f32;
            b[i * 2 + 1] = ((i * 2 + 1) as f64 * pi / (n * 2) as f64).sin() as f32;
        }
        let mut c = vec![0f32; quarter];
        for i in 0..eighth {
            c[i * 2] = ((i * 4 + 2) as f64 * pi / n as f64).cos() as f32;
            c[i * 2 + 1] = -(((i * 4 + 2) as f64 * pi / n as f64).sin() as f32);
        }
        let bits = ilog(eighth as i32 - 1);
        let reversed = (0..eighth).map(|i| reverse_bits(i, bits)).collect();
        BlockTables { a, b, c, reversed }
    }
}

/// `PlayerVarProvider.method285`.
fn reverse_bits(mut v: usize, bits: u32) -> usize {
    let mut out = 0;
    for _ in 0..bits {
        out = out << 1 | (v & 1);
        v >>= 1;
    }
    out
}

/// The shared setup (`MIDIInstrument.method12261`, file 0/0 of index 14).
pub struct Setup {
    short_block: usize,
    long_block: usize,
    tables: [BlockTables; 2],
    books: Vec<Codebook>,
    floors: Vec<Floor>,
    residues: Vec<Residue>,
    mappings: Vec<Mapping>,
    /// `aBoolArray7619` / `anIntArray7621`: per mode, long block and mapping.
    modes: Vec<(bool, usize)>,
}

impl Setup {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut b = Bits::new(data);
        let short_block = 1usize << b.bits(4);
        let long_block = 1usize << b.bits(4);
        let tables = [BlockTables::new(short_block), BlockTables::new(long_block)];
        let books = (0..b.bits(8) + 1).map(|_| Codebook::read(&mut b)).collect();
        for _ in 0..b.bits(6) + 1 {
            b.bits(16);
        }
        let floors = (0..b.bits(6) + 1).map(|_| Floor::read(&mut b)).collect::<Result<Vec<_>>>()?;
        let residues = (0..b.bits(6) + 1).map(|_| Residue::read(&mut b)).collect();
        let mappings = (0..b.bits(6) + 1).map(|_| Mapping::read(&mut b)).collect();
        let modes = (0..b.bits(6) + 1)
            .map(|_| {
                let long = b.bit() != 0;
                b.bits(16);
                b.bits(16);
                (long, b.bits(8) as usize)
            })
            .collect();
        Ok(Setup { short_block, long_block, tables, books, floors, residues, mappings, modes })
    }
}

/// A decoded instrument sample (`Node_Sub26_Sub1_Sub1`).
pub struct Sample {
    /// `anInt7605`.
    pub rate: i32,
    /// `anInt7616` / `anInt7604`: loop start and end.
    pub loop_start: i32,
    pub loop_end: i32,
    /// `aBool7609`: the stored loop end was negative (bit-inverted).
    pub loop_flag: bool,
    /// One float per sample, `anInt7620` of them.
    pub pcm: Vec<f32>,
}

/// One sample's packets (`method12265`) decoded in full (`method12273` up to the last sample).
pub fn decode_sample(setup: &Setup, data: &[u8]) -> Result<Sample> {
    let mut r = super::buffer::Reader::new(data);
    let rate = r.i32()?;
    let length = r.i32()?.max(0) as usize;
    let loop_start = r.i32()?;
    let mut loop_end = r.i32()?;
    let loop_flag = loop_end < 0;
    if loop_flag {
        loop_end = !loop_end;
    }
    let packet_count = r.i32()?;
    if packet_count < 0 {
        return corrupt("vorbis sample with negative packet count");
    }
    let mut packets = Vec::with_capacity(packet_count as usize);
    for _ in 0..packet_count {
        let mut size = 0usize;
        loop {
            let b = r.u8()?;
            size += b as usize;
            if b < 255 {
                break;
            }
        }
        packets.push(r.bytes(size)?);
    }
    let mut d = Decoder {
        setup,
        current: vec![0.0; setup.long_block],
        previous: vec![0.0; setup.long_block],
        previous_size: 0,
        previous_tail: 0,
        previous_skipped: false,
    };
    let mut pcm = Vec::with_capacity(length);
    for p in packets {
        if pcm.len() >= length {
            break;
        }
        if let Some(out) = d.packet(p) {
            let room = length - pcm.len();
            pcm.extend(out.into_iter().take(room));
        }
    }
    Ok(Sample { rate, loop_start, loop_end, loop_flag, pcm })
}

struct Decoder<'a> {
    setup: &'a Setup,
    /// `aFloatArray7612` and `aFloatArray7622`, swapped after each packet.
    current: Vec<f32>,
    previous: Vec<f32>,
    /// `anInt7623`, `anInt7624`, `aBool7625`.
    previous_size: usize,
    previous_tail: usize,
    previous_skipped: bool,
}

impl Decoder<'_> {
    /// `method12267`: one packet; returns the finished samples (none for the first).
    fn packet(&mut self, data: &[u8]) -> Option<Vec<f32>> {
        let s = self.setup;
        let mut b = Bits::new(data);
        b.bit();
        let mode = b.bits(ilog(s.modes.len() as i32 - 1)) as usize;
        let (long, mapping) = *s.modes.get(mode)?;
        let n = if long { s.long_block } else { s.short_block };
        let (mut prev_long, mut next_long) = (false, false);
        if long {
            prev_long = b.bit() != 0;
            next_long = b.bit() != 0;
        }
        let half = n >> 1;
        let short = s.short_block;
        let (left_start, left_end, left_size) =
            if long && !prev_long { ((n >> 2) - (short >> 2), (short >> 2) + (n >> 2), short >> 1) } else { (0, half, n >> 1) };
        let (right_start, right_end, right_size) =
            if long && !next_long { (n - (n >> 2) - (short >> 2), (short >> 2) + (n - (n >> 2)), short >> 1) } else { (half, n, n >> 1) };
        let m = &s.mappings[mapping];
        let floor = &s.floors[m.floors[m.floor_submap]];
        let floor_y = floor.decode(&mut b, &s.books);
        let skipped = floor_y.is_none();
        for &r in &m.residues {
            s.residues[r].decode(&mut b, &s.books, &mut self.current, n >> 1, skipped);
        }
        if let Some(y) = floor_y {
            floor.apply(y, &mut self.current, n >> 1);
        }
        if skipped {
            self.current[n >> 1..n].fill(0.0);
        } else {
            imdct(&mut self.current, n, &s.tables[usize::from(long)]);
            let pi = std::f64::consts::PI;
            for i in left_start..left_end {
                let f = ((((i - left_start) as f64 + 0.5) / left_size as f64 * 0.5 * pi).sin()) as f32;
                self.current[i] *= ((pi / 2.0) * f as f64 * f as f64).sin() as f32;
            }
            for i in right_start..right_end {
                let f = ((((i - right_start) as f64 + 0.5) / right_size as f64 * 0.5 * pi + pi / 2.0).sin()) as f32;
                self.current[i] *= ((pi / 2.0) * f as f64 * f as f64).sin() as f32;
            }
        }
        let mut out = None;
        if self.previous_size > 0 {
            let len = (n + self.previous_size) >> 2;
            let mut buf = vec![0f32; len];
            if !self.previous_skipped {
                let tail = &self.previous[self.previous_size >> 1..];
                for (v, p) in buf.iter_mut().zip(tail).take(self.previous_tail) {
                    *v += p;
                }
            }
            if !skipped {
                for i in left_start..n >> 1 {
                    let at = len - (n >> 1) + i;
                    buf[at] += self.current[i];
                }
            }
            out = Some(buf);
        }
        std::mem::swap(&mut self.current, &mut self.previous);
        self.previous_size = n;
        self.previous_tail = right_end - (n >> 1);
        self.previous_skipped = skipped;
        out
    }
}

/// The inverse MDCT of `method12267`, in place over `x[0..n]`.
fn imdct(x: &mut [f32], n: usize, t: &BlockTables) {
    let (n2, n4, n8) = (n >> 1, n >> 2, n >> 3);
    for v in x.iter_mut().take(n2) {
        *v *= 0.5;
    }
    for i in n2..n {
        x[i] = -x[n - i - 1];
    }
    let a = &t.a;
    for k in 0..n4 {
        let f27 = x[k * 4] - x[n - k * 4 - 1];
        let f28 = x[k * 4 + 2] - x[n - k * 4 - 3];
        let (f29, f30) = (a[k * 2], a[k * 2 + 1]);
        x[n - k * 4 - 1] = f27 * f29 - f28 * f30;
        x[n - k * 4 - 3] = f27 * f30 + f28 * f29;
    }
    for k in 0..n8 {
        let f27 = x[n2 + k * 4 + 3];
        let f28 = x[n2 + k * 4 + 1];
        let f29 = x[k * 4 + 3];
        let f30 = x[k * 4 + 1];
        x[n2 + k * 4 + 3] = f27 + f29;
        x[n2 + k * 4 + 1] = f28 + f30;
        let f31 = a[n2 - 4 - k * 4];
        let f32_ = a[n2 - 3 - k * 4];
        x[k * 4 + 3] = (f27 - f29) * f31 - (f28 - f30) * f32_;
        x[k * 4 + 1] = (f28 - f30) * f31 + (f27 - f29) * f32_;
    }
    let log = ilog(n as i32 - 1) as usize;
    for l in 0..log.saturating_sub(3) {
        let step = n >> (l + 2);
        let stride = 8 << l;
        for r in 0..2 << l {
            let i49 = n - step * r * 2;
            let i50 = n - step * (r * 2 + 1);
            for k in 0..n >> (l + 4) {
                let o = k * 4;
                let f35 = x[i49 - 1 - o];
                let f36 = x[i49 - 3 - o];
                let f37 = x[i50 - 1 - o];
                let f38 = x[i50 - 3 - o];
                x[i49 - 1 - o] = f35 + f37;
                x[i49 - 3 - o] = f36 + f38;
                let (f39, f40) = (a[k * stride], a[k * stride + 1]);
                x[i50 - 1 - o] = (f35 - f37) * f39 - (f36 - f38) * f40;
                x[i50 - 3 - o] = (f36 - f38) * f39 + (f35 - f37) * f40;
            }
        }
    }
    for i in 1..n8.saturating_sub(1) {
        let j = t.reversed[i];
        if i < j {
            let (p, q) = (i * 8, j * 8);
            for off in [1, 3, 5, 7] {
                x.swap(p + off, q + off);
            }
        }
    }
    for i in 0..n2 {
        x[i] = x[i * 2 + 1];
    }
    for i in 0..n8 {
        x[n - 1 - i * 2] = x[i * 4];
        x[n - 2 - i * 2] = x[i * 4 + 1];
        x[n - n4 - 1 - i * 2] = x[i * 4 + 2];
        x[n - n4 - 2 - i * 2] = x[i * 4 + 3];
    }
    let c = &t.c;
    for i in 0..n8 {
        let (f28, f29) = (c[i * 2], c[i * 2 + 1]);
        let f30 = x[n2 + i * 2];
        let f31 = x[n2 + i * 2 + 1];
        let f32_ = x[n - 2 - i * 2];
        let f51 = x[n - 1 - i * 2];
        let mut f52 = f29 * (f30 - f32_) + f28 * (f31 + f51);
        x[n2 + i * 2] = (f30 + f32_ + f52) * 0.5;
        x[n - 2 - i * 2] = (f30 + f32_ - f52) * 0.5;
        f52 = f29 * (f31 + f51) - f28 * (f30 - f32_);
        x[n2 + i * 2 + 1] = (f31 - f51 + f52) * 0.5;
        x[n - 1 - i * 2] = (-f31 + f51 + f52) * 0.5;
    }
    let bt = &t.b;
    for i in 0..n4 {
        x[i] = x[n2 + i * 2] * bt[i * 2] + x[n2 + i * 2 + 1] * bt[i * 2 + 1];
        x[n2 - 1 - i] = x[n2 + i * 2] * bt[i * 2 + 1] - x[n2 + i * 2 + 1] * bt[i * 2];
    }
    for i in 0..n4 {
        x[i + (n - n4)] = -x[i];
    }
    for i in 0..n4 {
        x[i] = x[n4 + i];
    }
    for i in 0..n4 {
        x[n4 + i] = -x[n4 - i - 1];
    }
    for i in 0..n4 {
        x[n2 + i] = x[n - i - 1];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_bits_least_significant_first() {
        let mut b = Bits::new(&[0b1010_1101, 0b0000_0011]);
        assert_eq!(b.bit(), 1);
        assert_eq!(b.bits(3), 0b110);
        assert_eq!(b.bits(6), 0b11_1010);
    }

    #[test]
    fn helpers_match_the_reference() {
        assert_eq!((ilog(0), ilog(1), ilog(7), ilog(8)), (0, 1, 3, 4));
        assert_eq!(pow_i(3, 4), 81);
        assert_eq!(lookup1_values(81, 4), 3);
        assert_eq!(reverse_bits(0b001, 3), 0b100);
        assert_eq!(unpack_float((788 << 21) | 3), 3.0);
    }
}
