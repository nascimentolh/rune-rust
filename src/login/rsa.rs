//! Textbook RSA as the client uses it: one modPow over the whole block.
//! Also generates new key pairs (`--gen-rsa`) so client and server can be matched.

use crate::config::Rsa;
use num_bigint::BigUint;
use rand::RngCore;

pub const PUBLIC_EXPONENT: u32 = 65537;
const MILLER_RABIN_ROUNDS: usize = 32;

/// `data ^ exponent mod modulus`, as bytes (no padding, like the client).
pub fn crypt(data: &[u8], key: &Rsa) -> Vec<u8> {
    BigUint::from_bytes_be(data).modpow(&key.exponent, &key.modulus).to_bytes_be()
}

fn random(bits: u64) -> BigUint {
    let mut bytes = vec![0u8; bits.div_ceil(8) as usize];
    rand::rng().fill_bytes(&mut bytes);
    bytes[0] |= 0xC0; // top two bits set, so p*q has the full size
    BigUint::from_bytes_be(&bytes)
}

fn is_probable_prime(n: &BigUint) -> bool {
    let one = BigUint::from(1u32);
    let two = BigUint::from(2u32);
    if n < &BigUint::from(4u32) {
        return n >= &two;
    }
    if !n.bit(0) {
        return false;
    }
    let n1 = n - &one;
    let s = n1.trailing_zeros().unwrap_or(0);
    let d = &n1 >> s;
    'witness: for _ in 0..MILLER_RABIN_ROUNDS {
        let a = random(n.bits() - 1) % (n - 3u32) + &two;
        let mut x = a.modpow(&d, n);
        if x == one || x == n1 {
            continue;
        }
        for _ in 1..s {
            x = x.modpow(&two, n);
            if x == n1 {
                continue 'witness;
            }
        }
        return false;
    }
    true
}

fn prime(bits: u64) -> BigUint {
    loop {
        let candidate = random(bits) | BigUint::from(1u32);
        if is_probable_prime(&candidate) {
            return candidate;
        }
    }
}

/// Returns (public key, private key) sharing one modulus.
pub fn generate(bits: u64) -> (Rsa, Rsa) {
    let e = BigUint::from(PUBLIC_EXPONENT);
    loop {
        let (p, q) = (prime(bits / 2), prime(bits / 2));
        let phi = (&p - 1u32) * (&q - 1u32);
        if let Some(d) = e.modinv(&phi) {
            let n = p * q;
            return (Rsa { exponent: e, modulus: n.clone() }, Rsa { exponent: d, modulus: n });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_keys_round_trip() {
        let (public, private) = generate(512);
        let msg = [10u8, 1, 2, 3, 4, 5];
        assert_eq!(crypt(&crypt(&msg, &public), &private), msg);
    }
}
