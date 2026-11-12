//! ISAAC random generator (Bob Jenkins), used after login to scramble packet opcodes.
//! The server reads with seeds as sent by the client and writes with every seed + 50.

const SIZE: usize = 256;
const GOLDEN: u32 = 0x9e37_79b9;

pub struct Isaac {
    mem: [u32; SIZE],
    rsl: [u32; SIZE],
    a: u32,
    b: u32,
    c: u32,
    count: usize,
}

impl Isaac {
    pub fn new(seed: &[i32]) -> Self {
        let mut s = Isaac { mem: [0; SIZE], rsl: [0; SIZE], a: 0, b: 0, c: 0, count: 0 };
        for (r, &v) in s.rsl.iter_mut().zip(seed) {
            *r = v as u32;
        }
        s.init();
        s
    }

    /// The pair a server needs: (reads from client, writes to client).
    pub fn pair(seed: [i32; 4]) -> (Isaac, Isaac) {
        (Isaac::new(&seed), Isaac::new(&seed.map(|v| v.wrapping_add(50))))
    }

    pub fn next(&mut self) -> u32 {
        if self.count == 0 {
            self.generate();
            self.count = SIZE;
        }
        self.count -= 1;
        self.rsl[self.count]
    }

    fn generate(&mut self) {
        self.c = self.c.wrapping_add(1);
        self.b = self.b.wrapping_add(self.c);
        for i in 0..SIZE {
            let x = self.mem[i];
            self.a ^= match i & 3 {
                0 => self.a << 13,
                1 => self.a >> 6,
                2 => self.a << 2,
                _ => self.a >> 16,
            };
            self.a = self.mem[(i + 128) & 0xff].wrapping_add(self.a);
            let y = self.mem[((x >> 2) & 0xff) as usize].wrapping_add(self.a).wrapping_add(self.b);
            self.mem[i] = y;
            self.b = self.mem[((y >> 10) & 0xff) as usize].wrapping_add(x);
            self.rsl[i] = self.b;
        }
    }

    fn init(&mut self) {
        let mut v = [GOLDEN; 8];
        for _ in 0..4 {
            mix(&mut v);
        }
        // First pass mixes in the seed, second pass mixes the result again.
        for pass in 0..2 {
            for i in (0..SIZE).step_by(8) {
                let source = if pass == 0 { &self.rsl[i..i + 8] } else { &self.mem[i..i + 8] };
                for (value, add) in v.iter_mut().zip(source) {
                    *value = value.wrapping_add(*add);
                }
                mix(&mut v);
                self.mem[i..i + 8].copy_from_slice(&v);
            }
        }
        self.generate();
        self.count = SIZE;
    }
}

fn mix(v: &mut [u32; 8]) {
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *v;
    a ^= b << 11;
    d = d.wrapping_add(a);
    b = b.wrapping_add(c);
    b ^= c >> 2;
    e = e.wrapping_add(b);
    c = c.wrapping_add(d);
    c ^= d << 8;
    f = f.wrapping_add(c);
    d = d.wrapping_add(e);
    d ^= e >> 16;
    g = g.wrapping_add(d);
    e = e.wrapping_add(f);
    e ^= f << 10;
    h = h.wrapping_add(e);
    f = f.wrapping_add(g);
    f ^= g >> 4;
    a = a.wrapping_add(f);
    g = g.wrapping_add(h);
    g ^= h << 8;
    b = b.wrapping_add(g);
    h = h.wrapping_add(a);
    h ^= a >> 9;
    c = c.wrapping_add(h);
    a = a.wrapping_add(b);
    *v = [a, b, c, d, e, f, g, h];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream_and_pair_differs() {
        let (mut read, mut write) = Isaac::pair([1, 2, 3, 4]);
        let mut again = Isaac::new(&[1, 2, 3, 4]);
        let first: Vec<u32> = (0..600).map(|_| read.next()).collect();
        assert!((0..600).all(|i| again.next() == first[i]));
        assert_ne!(write.next(), first[0]);
    }
}
