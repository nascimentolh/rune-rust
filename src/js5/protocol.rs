//! The JS5 wire format (cache update protocol), as used by build-718-era clients.
//! Written from public protocol knowledge — ESTIMATED, not checked against the
//! reference client (Clean room).
//!
//! Handshake (client → server, `jagex/client.java:2310-2320`): service 15 · size u8 ·
//! build 727 i32 · minor 1 i32 · key string (client parameter 17).
//! Reply (`client.java:2324-2350`): status u8 (0 ok, 6 out of date, 7/9 full, 48 session expired)
//! then, when ok, 27 i32 values (`Class446.method7436` has 27 entries).
//!
//! The rest below matches `jagex/JS5StandardRequester_Sub1.java:711-832`.
//! Reply: status u8 (0 = ok, 6 = out of date) · grab keys (i32 each, only when ok)
//!
//! Requests: 6 bytes each — opcode u8 then 5 bytes:
//!   0 / 1  archive request (1 = urgent): index u8 · archive i32
//!   2 / 3  player logged in / out        4  set XOR key: key u8     7  close
//!
//! Response: index u8 · archive i32 · compression u8 (| 0x80 if not urgent) · length i32 ·
//! container payload, with a 0xFF marker byte wherever the output offset reaches a multiple of 512.

use std::io::{self, Read};

pub const SERVICE_JS5: u8 = 15;
pub const STATUS_OK: u8 = 0;
pub const STATUS_OUT_OF_DATE: u8 = 6;
pub const REQUEST_LEN: usize = 6;
pub const BLOCK_SIZE: usize = 512;
pub const BLOCK_MARKER: u8 = 0xFF;
const RESPONSE_HEADER_LEN: usize = 10;
const NOT_URGENT_FLAG: u8 = 0x80;

#[derive(Debug, PartialEq)]
pub struct Handshake {
    pub build: i32,
    pub sub_build: Option<i32>,
    /// The key string after the builds, when the client sends one.
    pub key: Option<String>,
}

/// Reads the handshake body (after the service byte).
pub fn read_handshake(r: &mut impl Read) -> io::Result<Handshake> {
    let mut size = [0u8; 1];
    r.read_exact(&mut size)?;
    let mut body = vec![0u8; size[0] as usize];
    r.read_exact(&mut body)?;
    let int = |at: usize| body.get(at..at + 4).map(|b| i32::from_be_bytes(b.try_into().unwrap()));
    let build = int(0).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "handshake too short"))?;
    let key = body.get(8..).filter(|rest| !rest.is_empty()).map(|rest| rest.iter().take_while(|&&b| b != 0).map(|&b| b as char).collect());
    Ok(Handshake { build, sub_build: int(4), key })
}

/// The client's handshake: size = 9 + key length (two ints, the key and its end byte).
pub fn encode_handshake(build: i32, sub_build: i32, key: &str) -> Vec<u8> {
    let mut out = vec![SERVICE_JS5, (9 + key.len()) as u8];
    out.extend(build.to_be_bytes());
    out.extend(sub_build.to_be_bytes());
    out.extend(key.bytes());
    out.push(0);
    out
}

pub fn encode_handshake_ok(grab_keys: &[i32]) -> Vec<u8> {
    let mut out = vec![STATUS_OK];
    grab_keys.iter().for_each(|k| out.extend(k.to_be_bytes()));
    out
}

#[derive(Debug, PartialEq)]
pub enum Request {
    Archive { index: u8, archive: u32, urgent: bool },
    LoggedIn,
    LoggedOut,
    XorKey(u8),
    Close,
    Unknown(u8),
}

pub fn parse_request(b: [u8; REQUEST_LEN]) -> Request {
    match b[0] {
        0 | 1 => Request::Archive { index: b[1], archive: u32::from_be_bytes([b[2], b[3], b[4], b[5]]), urgent: b[0] == 1 },
        2 => Request::LoggedIn,
        3 => Request::LoggedOut,
        4 => Request::XorKey(b[1]),
        7 => Request::Close,
        other => Request::Unknown(other),
    }
}

pub fn encode_request(index: u8, archive: u32, urgent: bool) -> [u8; REQUEST_LEN] {
    let a = archive.to_be_bytes();
    [urgent as u8, index, a[0], a[1], a[2], a[3]]
}

/// Payload length of a stored container (after its 5-byte header, without version trailer).
fn payload_len(container: &[u8]) -> Option<usize> {
    let len = u32::from_be_bytes(container.get(1..5)?.try_into().ok()?) as usize;
    let real = if container[0] == 0 { len } else { len + 4 };
    (5 + real <= container.len()).then_some(real)
}

/// Wraps a stored container for sending. `None` if the container is malformed.
pub fn encode_response(index: u8, archive: u32, urgent: bool, container: &[u8]) -> Option<Vec<u8>> {
    let real = payload_len(container)?;
    let mut out = Vec::with_capacity(RESPONSE_HEADER_LEN + real + real / BLOCK_SIZE + 1);
    out.push(index);
    out.extend(archive.to_be_bytes());
    out.push(container[0] | if urgent { 0 } else { NOT_URGENT_FLAG });
    out.extend_from_slice(&container[1..5]);
    for &b in &container[5..5 + real] {
        if out.len() % BLOCK_SIZE == 0 {
            out.push(BLOCK_MARKER);
        }
        out.push(b);
    }
    Some(out)
}

/// Reads one response (client side, used by the self-test). Returns (index, archive, container).
pub fn read_response(r: &mut impl Read) -> io::Result<(u8, u32, Vec<u8>)> {
    let mut header = [0u8; RESPONSE_HEADER_LEN];
    r.read_exact(&mut header)?;
    let index = header[0];
    let archive = u32::from_be_bytes(header[1..5].try_into().unwrap());
    let compression = header[5] & !NOT_URGENT_FLAG;
    let len = u32::from_be_bytes(header[6..10].try_into().unwrap()) as usize;
    let mut remaining = if compression == 0 { len } else { len + 4 };

    let mut container = vec![compression];
    container.extend_from_slice(&header[6..10]);
    let mut offset = RESPONSE_HEADER_LEN;
    while remaining > 0 {
        if offset.is_multiple_of(BLOCK_SIZE) {
            let mut marker = [0u8; 1];
            r.read_exact(&mut marker)?;
            if marker[0] != BLOCK_MARKER {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "missing block marker"));
            }
            offset += 1;
        }
        let take = remaining.min(BLOCK_SIZE - offset % BLOCK_SIZE);
        let start = container.len();
        container.resize(start + take, 0);
        r.read_exact(&mut container[start..])?;
        offset += take;
        remaining -= take;
    }
    Ok((index, archive, container))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_round_trips_across_blocks() {
        for size in [0usize, 1, 501, 502, 503, 2000] {
            let mut container = vec![2];
            container.extend((size as u32).to_be_bytes());
            container.extend((0..size + 4).map(|i| i as u8));
            let wire = encode_response(7, 70_000, false, &container).unwrap();
            assert_eq!(wire[5] & 0x80, 0x80);
            let (index, archive, back) = read_response(&mut &wire[..]).unwrap();
            assert_eq!((index, archive), (7, 70_000));
            assert_eq!(back, container, "size {size}");
        }
    }

    #[test]
    fn handshake_and_requests() {
        let hs = encode_handshake(727, 1, "abc");
        assert_eq!(read_handshake(&mut &hs[1..]).unwrap(), Handshake { build: 727, sub_build: Some(1), key: Some("abc".into()) });
        assert_eq!(parse_request(encode_request(255, 255, true)), Request::Archive { index: 255, archive: 255, urgent: true });
        assert_eq!(parse_request([4, 9, 0, 0, 0, 0]), Request::XorKey(9));
    }
}
