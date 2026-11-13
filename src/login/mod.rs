//! The login service: what the client does right after the cache has loaded.
//!
//! - `accounts` — who may log in (first login creates the account)
//! - `lobby`    — what a lobby player is sent after the reply
//! - `world`    — the first game packet of a world login
//! - `world_list` — the world list the lobby is given
//! - here       — one client's login, then reading what it sends next and answering it
//!
//! The bytes on the wire (login packet, replies, ciphers, packet tables) are in
//! `crate::protocol`, shared with the client.

mod accounts;
mod lobby;
mod world;
mod world_list;

use crate::config::{Rsa, Settings};
use crate::net::{is_hang_up, Ctx};
use crate::players::World;
use crate::protocol::login::{self as protocol, Kind};
use crate::protocol::{client_prot, isaac, server_prot};
use std::io::{Read, Write};
use std::sync::Arc;

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
    /// The address the world list gives for this server's world.
    host: String,
    world: Arc<World>,
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
            host: settings.host.clone(),
            world: World::new(settings.world_capacity),
            accounts: accounts::Accounts::new(settings.accounts_file.clone()),
            map_keys,
        }
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
        // A world login needs a free player index; the lobby isn't a world and takes none.
        let seat = match req.kind {
            Kind::World => match self.world.join() {
                Some(seat) => Some(seat),
                None => {
                    ctx.log(format!("world full ({} players) - refused '{}'", self.world.capacity(), req.username));
                    return stream.write_all(&[protocol::WORLD_FULL]);
                }
            },
            Kind::Lobby => None,
        };
        if let Err(code) = self.accounts.log_in(&req.username, &req.password, req.kind == Kind::World) {
            ctx.log(format!("login as '{}' refused with reply {code}", req.username));
            return stream.write_all(&[code]);
        }

        let index = seat.as_ref().map_or(0, |s| s.index);
        match req.kind {
            Kind::World => stream.write_all(&protocol::encode_ok(RIGHTS_PLAYER, index, &req.username))?,
            Kind::Lobby => {
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64);
                stream.write_all(&protocol::encode_lobby_ok(&req.username, now + lobby::MEMBERSHIP_MS, now))?
            }
        }
        let (from_client, mut to_client) = isaac::Isaac::pair(req.seeds);
        // The game's window this player has open (none in the lobby).
        let window = (req.kind == Kind::World).then(|| world::game_window(req.display));
        // A world login needs its first game packet before the client shows the world; the
        // lobby gets what its window waits for.
        let packets = match window {
            // Varc 1413 first: the window's on-load scripts read it.
            Some(window) => [
                (server_prot::REGION, world::region_payload(index, world::SPAWN, &self.map_keys)),
                server_prot::varc(world::GAME_TABS_VARC.0, world::GAME_TABS_VARC.1),
                (server_prot::IF_OPENTOP, server_prot::open_top(window)),
            ]
            .into_iter()
            .chain(world::side_panels(window).into_iter().map(|p| (server_prot::IF_OPENSUB, p)))
            .collect(),
            None => lobby::first_packets(),
        };
        for (id, payload) in packets {
            stream.write_all(&server_prot::frame(id, &payload, &mut to_client))?;
        }
        ctx.stats.lock().unwrap().logins += 1;
        let place = match req.kind {
            Kind::Lobby => "the lobby".to_string(),
            Kind::World => format!("the world as player {index} of {} (display {:?})", self.world.online(), req.display),
        };
        ctx.log(format!("'{}' logged in to {place}, build {}.{}", req.username, req.build, req.sub_build));
        let result = self.after_login(stream, ctx, from_client, to_client, window);
        self.accounts.log_out(&req.username, req.kind == Kind::World);
        drop(seat);
        result
    }

    /// After login: reads the client's packets one by one (id unscrambled with the client's
    /// cipher, size from `ClientProt`), logs them and sends what answers them.
    fn after_login<S: Read + Write>(
        &self,
        stream: &mut S,
        ctx: &Ctx,
        mut from_client: isaac::Isaac,
        mut to_client: isaac::Isaac,
        mut window: Option<u16>,
    ) -> std::io::Result<()> {
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
            for (id, reply) in self.answer(id, &payload, &mut window) {
                stream.write_all(&server_prot::frame(id, &reply, &mut to_client))?;
            }
        }
    }

    /// The server's answer to a client packet, if it has one. `window` is the game's window the
    /// player has open.
    fn answer(&self, id: u32, payload: &[u8], window: &mut Option<u16>) -> Vec<(u32, Vec<u8>)> {
        match id {
            // A new window mode: the other game window with its side panels when it needs one
            // (this server's choice).
            client_prot::SCREEN_SIZE => {
                let (Some(open), Some(&mode)) = (window.as_mut(), payload.first()) else { return Vec::new() };
                let wanted = world::game_window(Some((mode, 0, 0)));
                if wanted == *open {
                    return Vec::new();
                }
                *open = wanted;
                std::iter::once((server_prot::IF_OPENTOP, server_prot::open_top(wanted)))
                    .chain(world::side_panels(wanted).into_iter().map(|p| (server_prot::IF_OPENSUB, p)))
                    .collect()
            }
            client_prot::REQUEST_WORLD_LIST => match payload.try_into() {
                Ok(held) => vec![(server_prot::WORLD_LIST, world_list::reply(i32::from_be_bytes(held), &self.host, &self.world))],
                Err(_) => Vec::new(),
            },
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::Stats;
    use crate::protocol::login::LoginRequest;
    use std::net::{TcpListener, TcpStream};
    use std::sync::Mutex;

    fn world_login(name: &str) -> Vec<u8> {
        let req = LoginRequest {
            kind: Kind::World,
            build: 727,
            sub_build: 1,
            reconnecting: false,
            seeds: [1, 2, 3, 4],
            password: "pw".into(),
            username: name.into(),
            display: Some((2, 765, 503)),
        };
        let mut out = vec![protocol::SERVICE_LOGIN];
        out.extend(protocol::encode(&req, None));
        out
    }

    /// `SCREEN_SIZE` with another mode opens the other game window, once; the lobby gets none.
    #[test]
    fn a_new_window_mode_opens_the_other_game_window() {
        let service = LoginService::new(&Settings { accounts_file: None, ..Settings::default() }, Default::default());
        let resizable = client_prot::screen_size(2, 1024, 768, 0);
        let fixed = client_prot::screen_size(1, 765, 503, 0);
        let mut window = Some(548);
        let opened = |packets: Vec<(u32, Vec<u8>)>| packets.first().map(|(id, p)| (*id, p[8], packets.len()));
        // 746 (0x02ea: bytes 2, 0xea + 128), then its 15 side panels.
        assert_eq!(opened(service.answer(client_prot::SCREEN_SIZE, &resizable, &mut window)), Some((server_prot::IF_OPENTOP, 2, 16)));
        assert_eq!(window, Some(746));
        assert!(service.answer(client_prot::SCREEN_SIZE, &resizable, &mut window).is_empty());
        assert_eq!(opened(service.answer(client_prot::SCREEN_SIZE, &fixed, &mut window)), Some((server_prot::IF_OPENTOP, 2, 16)));
        assert_eq!(window, Some(548));
        assert!(service.answer(client_prot::SCREEN_SIZE, &fixed, &mut None).is_empty());
    }

    /// 250 players log in to a world with room for 210 at the same time, over real
    /// connections: 210 get in with distinct indices, the rest are told the world is full, and
    /// every index is free again once they leave.
    #[test]
    fn a_full_world_refuses_the_rest_and_frees_seats_on_leave() {
        let settings = Settings { accounts_file: None, world_capacity: 210, ..Settings::default() };
        let service = Arc::new(LoginService::new(&settings, Default::default()));
        let stats = Arc::new(Mutex::new(Stats::default()));
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        {
            let (service, stats) = (Arc::clone(&service), Arc::clone(&stats));
            std::thread::spawn(move || {
                for mut stream in listener.incoming().flatten() {
                    let (service, stats) = (Arc::clone(&service), Arc::clone(&stats));
                    std::thread::spawn(move || {
                        let mut byte = [0u8; 1];
                        stream.read_exact(&mut byte).unwrap();
                        let _ = service.session(&mut stream, &Ctx { peer: "test", stats: &stats });
                    });
                }
            });
        }

        let clients: Vec<(TcpStream, u8)> = (0..250)
            .map(|i| {
                std::thread::spawn(move || {
                    let mut s = TcpStream::connect(addr).unwrap();
                    s.write_all(&world_login(&format!("player{i}"))).unwrap();
                    let mut reply = [0u8; 2];
                    s.read_exact(&mut reply).unwrap();
                    (s, reply[1])
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        let ok = clients.iter().filter(|(_, code)| *code == protocol::OK).count();
        let full = clients.iter().filter(|(_, code)| *code == protocol::WORLD_FULL).count();
        assert_eq!((ok, full), (210, 40));
        assert_eq!(service.world.online(), 210);

        drop(clients);
        for _ in 0..200 {
            if service.world.online() == 0 {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("{} seats still held after everyone left", service.world.online());
    }
}
