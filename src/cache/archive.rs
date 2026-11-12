//! Splitting a decompressed archive into its files.
//!
//! An archive holding one file is just that file. With several files the data
//! is written in chunks, and the archive ends with a table:
//!   for each chunk, for each file: size delta (i32)   ·   chunk count (u8)

use super::buffer::Reader;
use super::error::{corrupt, Result};

pub fn split(data: &[u8], file_count: usize) -> Result<Vec<Vec<u8>>> {
    if file_count <= 1 {
        return Ok(vec![data.to_vec()]);
    }
    let Some(&chunks) = data.last() else { return corrupt("empty multi-file archive") };
    let chunks = chunks as usize;
    let table_len = chunks * file_count * 4;
    let Some(table_start) = data.len().checked_sub(1 + table_len) else {
        return corrupt("archive too short for its size table");
    };

    let mut table = Reader::new(&data[table_start..data.len() - 1]);
    let mut sizes = vec![vec![0usize; file_count]; chunks];
    for chunk in sizes.iter_mut() {
        let mut size = 0i32;
        for s in chunk.iter_mut() {
            size = size.wrapping_add(table.i32()?);
            *s = size as usize;
        }
    }

    let mut body = Reader::new(&data[..table_start]);
    let mut files = vec![Vec::new(); file_count];
    for chunk in &sizes {
        for (file, &size) in files.iter_mut().zip(chunk) {
            file.extend_from_slice(body.bytes(size)?);
        }
    }
    Ok(files)
}

/// The inverse of `split`, writing everything in one chunk (used to build test caches).
pub fn join(files: &[Vec<u8>]) -> Vec<u8> {
    if files.len() == 1 {
        return files[0].clone();
    }
    let mut out: Vec<u8> = files.concat();
    let mut last = 0i32;
    for f in files {
        out.extend((f.len() as i32 - last).to_be_bytes());
        last = f.len() as i32;
    }
    out.push(1);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_then_split() {
        let files = vec![b"abc".to_vec(), Vec::new(), b"hello world".to_vec()];
        assert_eq!(split(&join(&files), 3).unwrap(), files);
    }

    #[test]
    fn two_chunks() {
        // File 0 = "ab"+"c", file 1 = "x"+"yz".
        let mut data = b"abxcyz".to_vec();
        for d in [2i32, -1, 1, 1] {
            data.extend(d.to_be_bytes());
        }
        data.push(2);
        assert_eq!(split(&data, 2).unwrap(), vec![b"abc".to_vec(), b"xyz".to_vec()]);
    }
}
