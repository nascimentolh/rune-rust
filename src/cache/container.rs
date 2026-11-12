//! Containers: the compressed wrapper around every archive.
//!
//! Layout: compression type (1 byte), compressed length (4), then
//! - type 0 (none):  the data
//! - type 1 (bzip2): uncompressed length (4) + bzip2 stream without its "BZh1" magic
//! - type 2 (gzip):  uncompressed length (4) + gzip stream
//!
//! An optional 2-byte version may follow.

use super::buffer::Reader;
use super::error::{corrupt, Result};
use super::xtea::{self, Key};
use std::io::Read;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Compression {
    None = 0,
    Bzip2 = 1,
    Gzip = 2,
}

const BZIP2_MAGIC: &[u8] = b"BZh1";

/// Length of the container itself (header + payload), without the version trailer.
/// This is the part the reference table's CRC covers.
pub fn container_len(raw: &[u8]) -> Result<usize> {
    let mut r = Reader::new(raw);
    let kind = r.u8()?;
    let len = r.u32()? as usize;
    Ok(if kind == 0 { 5 + len } else { 9 + len })
}

/// Decrypts (XTEA, everything after the 5-byte header) and then decompresses.
pub fn decrypt_and_decompress(raw: &[u8], key: &Key) -> Result<Vec<u8>> {
    if xtea::is_zero(key) {
        return decompress(raw);
    }
    let end = container_len(raw)?;
    if end > raw.len() {
        return corrupt("container shorter than its header says");
    }
    let mut copy = raw[..end].to_vec();
    xtea::decrypt(&mut copy[5..], key);
    decompress(&copy)
}

pub fn decompress(raw: &[u8]) -> Result<Vec<u8>> {
    let mut r = Reader::new(raw);
    let kind = r.u8()?;
    let len = r.u32()? as usize;
    if kind == Compression::None as u8 {
        return Ok(r.bytes(len)?.to_vec());
    }
    let expected = r.u32()? as usize;
    let payload = r.bytes(len)?;
    let mut out = Vec::with_capacity(expected);
    match kind {
        1 => {
            let stream = BZIP2_MAGIC.iter().chain(payload).copied().collect::<Vec<u8>>();
            bzip2::read::BzDecoder::new(&stream[..]).read_to_end(&mut out)?;
        }
        2 => {
            flate2::read::GzDecoder::new(payload).read_to_end(&mut out)?;
        }
        other => return corrupt(format!("unknown compression type {other}")),
    }
    if out.len() != expected {
        return corrupt(format!("decompressed {} bytes, header said {expected}", out.len()));
    }
    Ok(out)
}

/// Builds a container (used to write test caches).
pub fn compress(data: &[u8], kind: Compression) -> Vec<u8> {
    use std::io::Write;
    let payload = match kind {
        Compression::None => data.to_vec(),
        Compression::Bzip2 => {
            let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(1));
            e.write_all(data).unwrap();
            e.finish().unwrap()[BZIP2_MAGIC.len()..].to_vec()
        }
        Compression::Gzip => {
            let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            e.write_all(data).unwrap();
            e.finish().unwrap()
        }
    };
    let mut out = vec![kind as u8];
    out.extend((payload.len() as u32).to_be_bytes());
    if kind != Compression::None {
        out.extend((data.len() as u32).to_be_bytes());
    }
    out.extend(payload);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_compression_round_trips() {
        let data: Vec<u8> = (0..2000u32).map(|i| (i * 7 % 251) as u8).collect();
        for kind in [Compression::None, Compression::Bzip2, Compression::Gzip] {
            let c = compress(&data, kind);
            assert_eq!(container_len(&c).unwrap(), c.len());
            assert_eq!(decompress(&c).unwrap(), data, "{kind:?}");
        }
    }

    #[test]
    fn encrypted_container_needs_the_right_key() {
        let data = b"map square data, long enough to span a few xtea blocks".repeat(4);
        let key = [11, 22, -33, 44];
        let mut c = compress(&data, Compression::Gzip);
        xtea::encrypt(&mut c[5..], &key);
        assert_eq!(decrypt_and_decompress(&c, &key).unwrap(), data);
        assert!(decrypt_and_decompress(&c, &[1, 2, 3, 4]).is_err());
    }
}
