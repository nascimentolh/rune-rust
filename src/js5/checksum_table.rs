//! The checksum table: what the client gets for index 255, archive 255.
//! It lists every index's reference table so the client can tell which are out of date.
//!
//! Body: index count u8 · per index: CRC i32 · revision i32 · whirlpool (64 bytes)
//! then a 65-byte block (0 + whirlpool of the body), RSA-signed when a key is set.
//! Sent as an uncompressed container. ESTIMATED layout (Clean room).

use crate::cache::{self, container, FileStore, META_INDEX};
use crate::config::Rsa;
use num_bigint::BigUint;
use whirlpool::{Digest, Whirlpool};

pub struct IndexChecksum {
    pub crc: i32,
    pub revision: i32,
    pub whirlpool: [u8; 64],
}

pub fn whirlpool(data: &[u8]) -> [u8; 64] {
    Whirlpool::digest(data).into()
}

/// Checksums of every index's reference table, as stored in index 255.
pub fn index_checksums(store: &mut FileStore, revisions: &[Option<i32>]) -> Vec<IndexChecksum> {
    (0..store.index_count())
        .map(|i| match (store.read(META_INDEX, i as u32), revisions.get(i).copied().flatten()) {
            (Ok(raw), Some(revision)) => IndexChecksum { crc: crc32fast::hash(&raw) as i32, revision, whirlpool: whirlpool(&raw) },
            _ => IndexChecksum { crc: 0, revision: 0, whirlpool: [0; 64] },
        })
        .collect()
}

/// Builds the container sent for 255/255.
pub fn build(checksums: &[IndexChecksum], rsa: Option<&Rsa>) -> Vec<u8> {
    let mut body = vec![checksums.len() as u8];
    for c in checksums {
        body.extend(c.crc.to_be_bytes());
        body.extend(c.revision.to_be_bytes());
        body.extend(c.whirlpool);
    }
    let mut signature = vec![0u8];
    signature.extend(whirlpool(&body));
    if let Some(rsa) = rsa {
        signature = BigUint::from_bytes_be(&signature).modpow(&rsa.exponent, &rsa.modulus).to_bytes_be();
    }
    body.extend(signature);
    container::compress(&body, container::Compression::None)
}

/// Reads a checksum table back (client side, used by the self-test). Returns (crc, revision) per index.
pub fn parse(container_bytes: &[u8]) -> cache::Result<Vec<(i32, i32)>> {
    let body = container::decompress(container_bytes)?;
    let count = *body.first().unwrap_or(&0) as usize;
    let entry = 4 + 4 + 64;
    if body.len() < 1 + count * entry {
        return Err(cache::CacheError::Corrupt("checksum table too short".into()));
    }
    Ok((0..count)
        .map(|i| {
            let e = &body[1 + i * entry..];
            (i32::from_be_bytes(e[0..4].try_into().unwrap()), i32::from_be_bytes(e[4..8].try_into().unwrap()))
        })
        .collect())
}
