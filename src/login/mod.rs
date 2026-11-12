//! The login service: what the client does right after the cache has loaded.
//!
//! - `protocol` — the login packet and replies
//! - `rsa`      — decrypting the secret part of the login (and making keys)
//! - `isaac`    — the opcode scrambler both sides switch to after login
//! - here       — one client's login, then logging what it sends next
//!
//! - `accounts` — who may log in (first login creates the account)

pub mod accounts;
pub mod client_prot;
pub mod isaac;
mod lobby;
pub mod protocol;
pub mod rsa;
pub mod server_prot;
pub mod world;

use crate::config::{Rsa, Settings};
use crate::net::{is_hang_up, Ctx};
use protocol::Kind;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU16, Ordering};

const MAX_PLAYERS: u16 = 2000;
const RIGHTS_PLAYER: u8 = 0;

fn read_u8<S: Read>(stream: &mut S) -> std::io::Result<u8> {
    let mut b = [0u8; 1];
    stream.read_exact(&mut b)?;
    Ok(b[0])
}

/// A client packet for the log: its name, and for `IF_OPn` the component it was on.
fn describe_client_packet(id: u32, p: &[u8]) -> String {
    let name = client_prot::name_of(id);
    match client_prot::IF_OPS.iter().position(|&op| op == id) {
        Some(n) if p.len() == 8 => {
            let component = i32::from_le_bytes([p[0], p[1], p[2], p[3]]);
            format!("client: {name} (option {}) on component {}:{}", n + 1, component >> 16, component & 0xffff)
        }
        _ => format!("client: {name} ({} bytes)", p.len()),
    }
}

pub struct LoginService {
    rsa: Option<Rsa>,
    build: i32,
    check_build: bool,
    next_index: AtomicU16,
    accounts: accounts::Accounts,
    /// Map keys by region, for the `REGION` packet.
    map_keys: std::collections::HashMap<u32, crate::cache::xtea::Key>,
}

impl LoginService {
    pub fn new(settings: &Settings, map_keys: std::collections::HashMap<u32, crate::cache::xtea::Key>) -> Self {
        LoginService {
            rsa: settings.login_rsa.clone(),
            build: settings.build,
            check_build: settings.check_build,
            next_index: AtomicU16::new(1),
            accounts: accounts::Accounts::new(settings.accounts_file.clone()),
            map_keys,
        }
    }

    fn player_index(&self) -> u16 {
        self.next_index.fetch_add(1, Ordering::Relaxed) % MAX_PLAYERS + 1
    }

    /// Runs after the service byte (14) has been read.
    pub fn session<S: Read + Write>(&self, stream: &mut S, ctx: &Ctx) -> std::io::Result<()> {
        stream.write_all(&[0])?;
        let (kind, payload) = protocol::read_packet(stream)?;
        let req = match protocol::decode(kind, &payload, self.rsa.as_ref()) {
            Ok(r) => r,
            Err(e) => {
                ctx.log(format!("login type {kind} not understood: {e}"));
                return stream.write_all(&[protocol::BAD_SESSION]);
            }
        };
        if self.check_build && req.build != self.build {
            ctx.log(format!("login build {}, expected {} - sent 'out of date'", req.build, self.build));
            return stream.write_all(&[protocol::OUT_OF_DATE]);
        }
        if req.username.trim().is_empty() || req.password.is_empty() {
            ctx.log("login with empty name or password - refused");
            return stream.write_all(&[protocol::INVALID_LOGIN]);
        }
        if let Err(code) = self.accounts.log_in(&req.username, &req.password) {
            ctx.log(format!("login as '{}' refused with reply {code}", req.username));
            return stream.write_all(&[code]);
        }

        let index = self.player_index();
        match req.kind {
            Kind::World => stream.write_all(&protocol::encode_ok(RIGHTS_PLAYER, index, &req.username))?,
            Kind::Lobby => {
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64);
                stream.write_all(&protocol::encode_lobby_ok(&req.username, now + lobby::MEMBERSHIP_MS, now))?
            }
        }
        let (from_client, mut to_client) = isaac::Isaac::pair(req.seeds);
        // A world login needs its first game packet before the client shows the world; the
        // lobby gets what its window waits for.
        let packets = match req.kind {
            Kind::World => vec![(world::REGION, world::region_payload(index, world::SPAWN, &self.map_keys))],
            Kind::Lobby => lobby::first_packets(),
        };
        for (id, payload) in packets {
            stream.write_all(&server_prot::frame(id, &payload, &mut to_client))?;
        }
        ctx.stats.lock().unwrap().logins += 1;
        let place = match req.kind {
            Kind::Lobby => "the lobby".to_string(),
            Kind::World => format!("the world as player {index} (display {:?})", req.display),
        };
        ctx.log(format!("'{}' logged in to {place}, build {}.{}", req.username, req.build, req.sub_build));
        let result = self.after_login(stream, ctx, from_client);
        self.accounts.log_out(&req.username);
        result
    }

    /// After login: reads the client's packets one by one (id unscrambled with the client's
    /// cipher, size from `ClientProt`) and logs them; none is acted on yet.
    fn after_login<S: Read>(&self, stream: &mut S, ctx: &Ctx, mut from_client: isaac::Isaac) -> std::io::Result<()> {
        loop {
            let mut id = [0u8; 1];
            match stream.read_exact(&mut id) {
                Ok(()) => {}
                Err(e) if is_hang_up(&e) || e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
                Err(e) => return Err(e),
            }
            let id = id[0].wrapping_sub(from_client.next() as u8) as u32;
            let Some(size) = client_prot::size_of(id) else {
                ctx.log(format!("client packet {id} isn't in ClientProt; stopped reading"));
                return Ok(());
            };
            let len = match size {
                -1 => read_u8(stream)? as usize,
                -2 => (read_u8(stream)? as usize) << 8 | read_u8(stream)? as usize,
                n => n as usize,
            };
            let mut payload = vec![0u8; len];
            stream.read_exact(&mut payload)?;
            ctx.log(describe_client_packet(id, &payload));
        }
    }
}
