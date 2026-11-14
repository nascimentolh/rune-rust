//! A logged-in client's packets, as the lobby and the world both read them: the id unscrambled
//! with the client's cipher, the size from `ClientProt`, then the payload.

use super::net::is_hang_up;
use crate::protocol::client_packets;
use crate::protocol::client_prot::{ClientProt, Packet};
use crate::protocol::isaac::Isaac;
use crate::protocol::packet::Size;
use std::io::{Error, ErrorKind, Read, Result};

fn read_u8<S: Read>(stream: &mut S) -> Result<u8> {
    let mut b = [0u8; 1];
    stream.read_exact(&mut b)?;
    Ok(b[0])
}

/// Waits for the client's next packet. `None` when the client has hung up between packets; an
/// id that isn't in `ClientProt` is an error (the stream can't be followed past it).
pub fn read_packet<S: Read>(stream: &mut S, from_client: &mut Isaac) -> Result<Option<Packet>> {
    let id = match read_u8(stream) {
        Ok(byte) => byte.wrapping_sub(from_client.next() as u8) as u32,
        Err(e) if is_hang_up(&e) => return Ok(None),
        Err(e) => return Err(e),
    };
    let packet =
        ClientProt::from_id(id).ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("client packet {id} isn't in ClientProt")))?;
    let len = match packet.size() {
        Size::Byte => read_u8(stream)? as usize,
        Size::Short => (read_u8(stream)? as usize) << 8 | read_u8(stream)? as usize,
        Size::Fixed(n) => n,
    };
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload)?;
    Ok(Some((packet, payload)))
}

/// A client packet for the log: its name, and for `IF_OPn` the component it was on.
pub fn describe(packet: ClientProt, payload: &[u8]) -> String {
    let name = packet.name();
    match (ClientProt::IF_OPS.iter().position(|&op| op == packet), client_packets::if_op_component(payload)) {
        (Some(n), Some(component)) => {
            format!("client: {name} (option {}) on component {}:{}", n + 1, component >> 16, component & 0xffff)
        }
        _ => format!("client: {name} ({} bytes)", payload.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::client_prot;

    #[test]
    fn packets_are_read_back_as_the_client_framed_them() {
        let (mut to_server, _) = Isaac::pair([5, 6, 7, 8]);
        let (mut from_client, _) = Isaac::pair([5, 6, 7, 8]);
        let sent = [client_packets::screen_size(2, 1024, 768, 0), client_packets::if_op(1, 182 << 16 | 6, -1, -1)];
        let wire: Vec<u8> = sent.iter().flat_map(|(packet, payload)| client_prot::frame(*packet, payload, &mut to_server)).collect();
        let mut stream = &wire[..];
        assert_eq!(read_packet(&mut stream, &mut from_client).unwrap(), Some(sent[0].clone()));
        let second = read_packet(&mut stream, &mut from_client).unwrap().unwrap();
        assert_eq!(describe(second.0, &second.1), "client: IF_OP1 (option 1) on component 182:6");
        // The client hangs up between packets.
        assert_eq!(read_packet(&mut stream, &mut from_client).unwrap(), None);
    }

    #[test]
    fn an_id_outside_the_table_stops_the_reading() {
        let (mut from_client, _) = Isaac::pair([0; 4]);
        let (mut same, _) = Isaac::pair([0; 4]);
        let wire = [200u8.wrapping_add(same.next() as u8)];
        assert!(read_packet(&mut &wire[..], &mut from_client).is_err());
    }
}
