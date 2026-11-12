//! The login service: what the client does right after the cache has loaded.
//!
//! - `protocol` — the login packet and replies
//! - `rsa`      — decrypting the secret part of the login (and making keys)
//! - `isaac`    — the opcode scrambler both sides switch to after login
//! - here       — one client's login, then logging what it sends next
//!
//! - `accounts` — who may log in (first login creates the account)

pub mod accounts;
pub mod isaac;
pub mod protocol;
pub mod rsa;
pub mod world;

use crate::config::{Rsa, Settings};
use crate::net::{is_hang_up, Ctx};
use protocol::Kind;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU16, Ordering};

const MAX_PLAYERS: u16 = 2000;
const RIGHTS_PLAYER: u8 = 0;

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
        stream.write_all(&protocol::encode_ok(RIGHTS_PLAYER, index, &req.username))?;
        let (from_client, mut to_client) = isaac::Isaac::pair(req.seeds);
        // A world login needs its first game packet before the client shows the world.
        if req.kind == Kind::World {
            let payload = world::region_payload(index, world::SPAWN, &self.map_keys);
            stream.write_all(&world::frame(world::REGION, &payload, &mut to_client))?;
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

    /// What comes after login isn't built yet. Log what the client sends; the
    /// first byte is decoded with ISAAC so the trace shows its first packet opcode.
    // ponytail: only the first opcode is decoded; later ones need the packet size table to find packet boundaries.
    fn after_login<S: Read>(&self, stream: &mut S, ctx: &Ctx, mut from_client: isaac::Isaac) -> std::io::Result<()> {
        let mut buf = [0u8; 4096];
        let mut first = true;
        loop {
            let n = match stream.read(&mut buf) {
                Ok(0) => return Ok(()),
                Ok(n) => n,
                Err(e) if is_hang_up(&e) => return Ok(()),
                Err(e) => return Err(e),
            };
            if first {
                let opcode = buf[0].wrapping_sub(from_client.next() as u8);
                ctx.log(format!("first game packet: opcode {opcode} (game packets not built yet)"));
                first = false;
            }
            ctx.log(format!("sent {n} bytes after login"));
        }
    }
}
