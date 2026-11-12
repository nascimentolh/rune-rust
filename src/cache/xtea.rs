//! XTEA block cipher (32 rounds, big-endian 8-byte blocks), used to encrypt
//! map archives. Bytes after the last whole block are left as they are.

pub type Key = [i32; 4];

const DELTA: u32 = 0x9E37_79B9;
const ROUNDS: u32 = 32;
const BLOCK: usize = 8;

pub fn is_zero(key: &Key) -> bool {
    key.iter().all(|&k| k == 0)
}

fn split(block: &[u8]) -> (u32, u32) {
    (u32::from_be_bytes(block[0..4].try_into().unwrap()), u32::from_be_bytes(block[4..8].try_into().unwrap()))
}

fn join(block: &mut [u8], v0: u32, v1: u32) {
    block[0..4].copy_from_slice(&v0.to_be_bytes());
    block[4..8].copy_from_slice(&v1.to_be_bytes());
}

pub fn decrypt(data: &mut [u8], key: &Key) {
    let k = key.map(|x| x as u32);
    for block in data.chunks_exact_mut(BLOCK) {
        let (mut v0, mut v1) = split(block);
        let mut sum = DELTA.wrapping_mul(ROUNDS);
        for _ in 0..ROUNDS {
            v1 = v1.wrapping_sub((((v0 << 4) ^ (v0 >> 5)).wrapping_add(v0)) ^ sum.wrapping_add(k[((sum >> 11) & 3) as usize]));
            sum = sum.wrapping_sub(DELTA);
            v0 = v0.wrapping_sub((((v1 << 4) ^ (v1 >> 5)).wrapping_add(v1)) ^ sum.wrapping_add(k[(sum & 3) as usize]));
        }
        join(block, v0, v1);
    }
}

pub fn encrypt(data: &mut [u8], key: &Key) {
    let k = key.map(|x| x as u32);
    for block in data.chunks_exact_mut(BLOCK) {
        let (mut v0, mut v1) = split(block);
        let mut sum = 0u32;
        for _ in 0..ROUNDS {
            v0 = v0.wrapping_add((((v1 << 4) ^ (v1 >> 5)).wrapping_add(v1)) ^ sum.wrapping_add(k[(sum & 3) as usize]));
            sum = sum.wrapping_add(DELTA);
            v1 = v1.wrapping_add((((v0 << 4) ^ (v0 >> 5)).wrapping_add(v0)) ^ sum.wrapping_add(k[((sum >> 11) & 3) as usize]));
        }
        join(block, v0, v1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_leaves_tail() {
        let key = [1, -2, 3, i32::MIN];
        let original: Vec<u8> = (0..21).collect();
        let mut data = original.clone();
        encrypt(&mut data, &key);
        assert_ne!(data[..16], original[..16]);
        assert_eq!(data[16..], original[16..]);
        decrypt(&mut data, &key);
        assert_eq!(data, original);
    }
}
