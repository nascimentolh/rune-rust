//! The game connection after a lobby or world login: it stays open, and every client tick the
//! packets that have arrived are read and applied (`PacketDecoder`).
//!
//! A packet: its id as an ISAAC-scrambled smart (`readEncryptedSmart`, the server's cipher is
//! the login seeds + 50), its length when `ServerProt` says -1 (u8) or -2 (u16), the payload.
//! Applied so far: `VARP_SMALL`, `VARP_LARGE`, `WORLD_LIST`, `IF_OPENTOP`, `IF_OPENSUB`. Others
//! are skipped by their size and logged.

use crate::client::interface::Ui;
use crate::protocol::client_prot;
use crate::protocol::isaac::Isaac;
use crate::protocol::server_prot::{
    self, CLIENT_SETVARC_LARGE, CLIENT_SETVARC_SMALL, IF_OPENSUB, IF_OPENTOP, LOGOUT_FULL, LOGOUT_LOBBY, RUN_ENERGY, UPDATE_STAT,
    VARP_LARGE, VARP_SMALL, WORLD_LIST,
};
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;

pub struct LobbyConnection {
    stream: TcpStream,
    /// Boxed: the cipher's state is about 2 KB.
    from_server: Box<Isaac>,
    /// The client's own cipher (the login seeds), for the packets it sends.
    to_server: Box<Isaac>,
    /// Bytes received and not yet used.
    pending: Vec<u8>,
    /// The first byte of a two-byte id, already unscrambled, waiting for the second.
    id_high: Option<u32>,
    /// A packet whose id was read, waiting for the rest: (id, size; -1/-2 until its length is in).
    current: Option<(u32, i32)>,
}

impl LobbyConnection {
    /// The connection the lobby login used, with the server's cipher (seeds + 50).
    pub fn new(stream: TcpStream, seeds: [i32; 4]) -> std::io::Result<Self> {
        let (to_server, from_server) = Isaac::pair(seeds);
        Self::resume(stream, to_server, from_server)
    }

    /// A world login's connection after its first packet (`REGION`), read with `from_server`.
    pub fn resume(stream: TcpStream, to_server: Isaac, from_server: Isaac) -> std::io::Result<Self> {
        stream.set_nonblocking(true)?;
        stream.set_read_timeout(None)?;
        Ok(LobbyConnection {
            stream,
            from_server: Box::new(from_server),
            to_server: Box::new(to_server),
            pending: Vec::new(),
            id_high: None,
            current: None,
        })
    }

    /// Sends the client's queued packets, then reads what has arrived and applies every whole
    /// packet. `Err` when the server closed the connection or sent something unreadable.
    pub fn poll(&mut self, ui: &mut Ui) -> Result<(), String> {
        for (id, payload) in std::mem::take(&mut ui.outgoing) {
            let wire = client_prot::frame(id, &payload, &mut self.to_server);
            self.stream.write_all(&wire).map_err(|e| e.to_string())?;
        }
        let mut buf = [0u8; 4096];
        let mut closed = false;
        loop {
            match self.stream.read(&mut buf) {
                Ok(0) => {
                    closed = true;
                    break;
                }
                Ok(n) => self.pending.extend_from_slice(&buf[..n]),
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) => return Err(e.to_string()),
            }
        }
        // What arrived before the server closed still applies (a logout comes last).
        while let Some((id, payload)) = self.next_packet()? {
            apply(ui, id, &payload);
        }
        if closed {
            return Err("the server closed the connection".into());
        }
        Ok(())
    }

    /// The next whole packet from `pending`, if one is there.
    fn next_packet(&mut self) -> Result<Option<(u32, Vec<u8>)>, String> {
        if self.current.is_none() {
            let Some(id) = self.read_id() else { return Ok(None) };
            let size = server_prot::size_of(id).ok_or(format!("unknown server packet {id}"))?;
            self.current = Some((id, size));
        }
        let Some((id, size)) = self.current else { return Ok(None) };
        let (len_bytes, len) = match size {
            -1 => match self.pending.first() {
                Some(&n) => (1, n as usize),
                None => return Ok(None),
            },
            -2 => match self.pending.get(..2) {
                Some(b) => (2, u16::from_be_bytes([b[0], b[1]]) as usize),
                None => return Ok(None),
            },
            n => (0, n as usize),
        };
        if self.pending.len() < len_bytes + len {
            return Ok(None);
        }
        let payload = self.pending[len_bytes..len_bytes + len].to_vec();
        self.pending.drain(..len_bytes + len);
        self.current = None;
        Ok(Some((id, payload)))
    }

    /// `readEncryptedSmart`, one byte at a time: each byte is unscrambled as it is taken, so a
    /// two-byte id split across reads keeps its first half in `id_high`.
    fn read_id(&mut self) -> Option<u32> {
        let high = match self.id_high.take() {
            Some(high) => high,
            None => {
                let first = self.take_byte()? as u32;
                if first < 128 {
                    return Some(first);
                }
                first - 128
            }
        };
        match self.take_byte() {
            Some(low) => Some((high << 8) + low as u32),
            None => {
                self.id_high = Some(high);
                None
            }
        }
    }

    /// The next byte, unscrambled with the server's cipher.
    fn take_byte(&mut self) -> Option<u8> {
        if self.pending.is_empty() {
            return None;
        }
        let b = self.pending.remove(0);
        Some(b.wrapping_sub(self.from_server.next() as u8))
    }
}

/// `PacketDecoder`'s handling of one packet.
fn apply(ui: &mut Ui, id: u32, p: &[u8]) {
    // `readUnsignedShortLE128`: low byte − 128, then the high byte.
    let id_le128 = |at: usize| (p[at].wrapping_sub(128) as i32) | (p[at + 1] as i32) << 8;
    match id {
        VARP_SMALL if p.len() == 3 => {
            ui.set_varp(id_le128(1), p[0] as i8 as i32);
        }
        // `readIntV2`: bytes 2, 1, 4, 3 of the value.
        VARP_LARGE if p.len() == 6 => {
            let value = i32::from_be_bytes([p[1], p[0], p[3], p[2]]);
            ui.set_varp(id_le128(4), value);
        }
        // `CLIENT_SETVARC_SMALL`: value `read128Byte` ((byte) (128 − b)), id `readShortLE`;
        // `PulseEvent.method6751` → `VARC_INT[id]` (varc change triggers aren't ported).
        CLIENT_SETVARC_SMALL if p.len() == 3 => {
            ui.varc.insert(i32::from(u16::from_le_bytes([p[1], p[2]])), 128u8.wrapping_sub(p[0]) as i8 as i32);
        }
        // `CLIENT_SETVARC_LARGE`: id `readShortLE`, value `readIntV2` (bytes 2, 1, 4, 3).
        CLIENT_SETVARC_LARGE if p.len() == 6 => {
            ui.varc.insert(i32::from(u16::from_le_bytes([p[0], p[1]])), i32::from_be_bytes([p[3], p[2], p[5], p[4]]));
        }
        // `RUN_ENERGY`: `client.RUN_ENERGY = readUnsignedByte`, stamped for the hooks.
        RUN_ENERGY if p.len() == 1 => ui.set_run_energy(i32::from(p[0])),
        // `UPDATE_STAT`: experience `readInt`, skill and level `readUnsignedByte`.
        UPDATE_STAT if p.len() == 6 => {
            let xp = i32::from_be_bytes([p[0], p[1], p[2], p[3]]);
            ui.set_skill(usize::from(p[4]), xp, i32::from(p[5]));
        }
        LOGOUT_FULL => ui.server_logout = Some(false),
        LOGOUT_LOBBY => ui.server_logout = Some(true),
        // `IF_OPENTOP`: the window id is the 9th and 10th bytes (u16 +128 on the low byte).
        IF_OPENTOP if p.len() == 19 => {
            let id = (p[8] as u32) << 8 | p[9].wrapping_sub(128) as u32;
            ui.open_top(id);
        }
        // `IF_OPENSUB`: the interface (u16 LE +128) at 12, the parent component (int) at 18.
        IF_OPENSUB if p.len() == 23 => {
            let id = p[12].wrapping_sub(128) as u32 | (p[13] as u32) << 8;
            let parent = u32::from_be_bytes([p[18], p[19], p[20], p[21]]);
            ui.open_sub_from_server(parent, id);
        }
        WORLD_LIST => {
            if let Err(e) = ui.worlds.receive(p, std::time::Instant::now()) {
                eprintln!("lobby: world list not understood: {e}");
            }
        }
        _ => eprintln!("lobby: server packet {id} ({} bytes) not handled yet", p.len()),
    }
}
