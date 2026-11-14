//! The login service: what the client does right after the cache has loaded.
//!
//! - `accounts` — who may log in (first login creates the account)
//! - here       — one client's login: read it, check it, reply, then hand the connection to
//!   the lobby (`super::lobby`) or to the world (`super::world`)
//!
//! Nothing after the reply is decided here. The bytes on the wire (login packet, replies,
//! ciphers) are in `crate::protocol`, shared with the client.

mod accounts;

pub use accounts::Online;

use super::lobby::{self, Lobby};
use super::net::{Connection, Ctx};
use super::world::{Arrival, World};
use crate::config::{Rsa, Settings};
use crate::protocol::isaac::Isaac;
use crate::protocol::login::{self as protocol, Kind};
use std::io::Write;
use std::sync::Arc;

const RIGHTS_PLAYER: u8 = 0;

pub struct LoginService {
    rsa: Option<Rsa>,
    build: i32,
    check_build: bool,
    accounts: Arc<accounts::Accounts>,
    lobby: Lobby,
    world: World,
}

impl LoginService {
    pub fn new(settings: &Settings, world: World) -> Self {
        LoginService {
            rsa: settings.login_rsa.clone(),
            build: settings.build,
            check_build: settings.check_build,
            accounts: accounts::Accounts::new(settings.accounts_file.clone()),
            lobby: Lobby::new(settings.host.clone(), Arc::clone(world.seats())),
            world,
        }
    }

    /// Runs after the service byte (14) has been read, for as long as the player stays.
    pub fn session(&self, stream: &mut Connection, ctx: &Ctx) -> std::io::Result<()> {
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
        let seats = self.world.seats();
        let seat = match req.kind {
            Kind::World => match seats.join() {
                Some(seat) => Some(seat),
                None => {
                    ctx.log(format!("world full ({} players) - refused '{}'", seats.capacity(), req.username));
                    return stream.write_all(&[protocol::WORLD_FULL]);
                }
            },
            Kind::Lobby => None,
        };
        let online = match self.accounts.log_in(&req.username, &req.password, req.kind == Kind::World) {
            Ok(online) => online,
            Err(code) => {
                ctx.log(format!("login as '{}' refused with reply {code}", req.username));
                return stream.write_all(&[code]);
            }
        };

        let reply = match &seat {
            Some(seat) => protocol::encode_ok(RIGHTS_PLAYER, seat.index, &req.username),
            None => {
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64);
                protocol::encode_lobby_ok(&req.username, now + lobby::MEMBERSHIP_MS, now)
            }
        };
        stream.write_all(&reply)?;
        ctx.stats.lock().unwrap().logins += 1;
        let place = match &seat {
            None => "the lobby".to_string(),
            Some(seat) => format!("the world as player {} of {} (display {:?})", seat.index, seats.online(), req.display),
        };
        ctx.log(format!("'{}' logged in to {place}, build {}.{}", req.username, req.build, req.sub_build));

        // (the client's cipher, the server's): both sides switch to them from here on.
        let ciphers = Isaac::pair(req.seeds);
        match seat {
            Some(seat) => {
                let arrival = Arrival { name: req.username, window_mode: req.display.map(|(mode, _, _)| mode), seat, online };
                self.world.play(stream, ctx, arrival, ciphers)
            }
            // `online` is held until the lobby session ends.
            None => self.lobby.session(stream, ctx, ciphers),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::client_packets;
    use crate::protocol::client_prot;
    use crate::protocol::login::LoginRequest;
    use crate::protocol::packet::Size;
    use crate::protocol::server_prot::ServerProt;
    use crate::server::net::Stats;
    use crate::server::trace::Traced;
    use std::io::Read;
    use std::net::{SocketAddr, TcpListener, TcpStream};
    use std::sync::Mutex;
    use std::time::Duration;

    const SEEDS: [i32; 4] = [1, 2, 3, 4];

    fn world_login(name: &str) -> Vec<u8> {
        let req = LoginRequest {
            kind: Kind::World,
            build: 727,
            sub_build: 1,
            reconnecting: false,
            seeds: SEEDS,
            password: "pw".into(),
            username: name.into(),
            display: Some((2, 765, 503)),
        };
        let mut out = vec![protocol::SERVICE_LOGIN];
        out.extend(protocol::encode(&req, None));
        out
    }

    /// A login service with a running world for `capacity` players, behind a real port.
    fn serve(capacity: u16) -> (Arc<LoginService>, SocketAddr) {
        let settings = Settings { accounts_file: None, ..Settings::default() };
        let stats = Arc::new(Mutex::new(Stats::default()));
        let service = Arc::new(LoginService::new(&settings, World::start(capacity, Default::default(), Arc::clone(&stats))));
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let serving = Arc::clone(&service);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let (service, stats) = (Arc::clone(&serving), Arc::clone(&stats));
                std::thread::spawn(move || {
                    let mut stream = Traced::new(stream, None, "test");
                    let mut byte = [0u8; 1];
                    stream.read_exact(&mut byte).unwrap();
                    let _ = service.session(&mut stream, &Ctx { peer: "test".into(), stats });
                });
            }
        });
        (service, addr)
    }

    /// Connects and logs in to the world; gives the connection and the reply code.
    fn log_in(addr: SocketAddr, name: &str) -> (TcpStream, u8) {
        let mut s = TcpStream::connect(addr).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(20))).unwrap();
        s.write_all(&world_login(name)).unwrap();
        let mut reply = [0u8; 2];
        s.read_exact(&mut reply).unwrap();
        (s, reply[1])
    }

    fn wait_until_empty(service: &LoginService) {
        for _ in 0..500 {
            if service.world.seats().online() == 0 {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("{} seats still held after everyone left", service.world.seats().online());
    }

    /// Reads server packets as a client does, until `wanted` has come.
    fn read_until(s: &mut TcpStream, from_server: &mut Isaac, wanted: ServerProt) {
        let byte = |s: &mut TcpStream| {
            let mut b = [0u8; 1];
            s.read_exact(&mut b).unwrap();
            b[0]
        };
        loop {
            let mut id = byte(s).wrapping_sub(from_server.next() as u8) as u32;
            if id >= 128 {
                id = (id - 128) << 8 | byte(s).wrapping_sub(from_server.next() as u8) as u32;
            }
            let packet = ServerProt::from_id(id).unwrap();
            let len = match packet.size() {
                Size::Fixed(n) => n,
                Size::Byte => byte(s) as usize,
                Size::Short => (byte(s) as usize) << 8 | byte(s) as usize,
            };
            s.read_exact(&mut vec![0u8; len]).unwrap();
            if packet == wanted {
                return;
            }
        }
    }

    /// 250 players log in to a world with room for 210 at the same time, over real
    /// connections: 210 get in with distinct indices, the rest are told the world is full, and
    /// every index is free again once they leave.
    #[test]
    fn a_full_world_refuses_the_rest_and_frees_seats_on_leave() {
        let (service, addr) = serve(210);
        let clients: Vec<(TcpStream, u8)> = (0..250)
            .map(|i| std::thread::spawn(move || log_in(addr, &format!("player{i}"))))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        let ok = clients.iter().filter(|(_, code)| *code == protocol::OK).count();
        let full = clients.iter().filter(|(_, code)| *code == protocol::WORLD_FULL).count();
        assert_eq!((ok, full), (210, 40));
        assert_eq!(service.world.seats().online(), 210);

        drop(clients);
        wait_until_empty(&service);
    }

    /// 250 players in one world, each on a real connection, all at once: every one is sent the
    /// world, has a packet answered by the tick, logs out through the tick and is let go.
    #[test]
    fn a_world_of_250_players_is_served_by_the_tick() {
        let (service, addr) = serve(250);
        let players: Vec<_> = (0..250)
            .map(|i| {
                std::thread::spawn(move || {
                    let (mut s, code) = log_in(addr, &format!("player{i}"));
                    assert_eq!(code, protocol::OK);
                    let mut size = [0u8; 1];
                    s.read_exact(&mut size).unwrap();
                    s.read_exact(&mut vec![0u8; size[0] as usize]).unwrap();
                    let (mut to_server, mut from_server) = Isaac::pair(SEEDS);
                    // The welcome is the last of what a player entering is sent.
                    read_until(&mut s, &mut from_server, ServerProt::GameMessage);
                    // Logged in as resizable: the fixed mode opens the other window, on a tick.
                    let (packet, payload) = client_packets::screen_size(1, 765, 503, 0);
                    s.write_all(&client_prot::frame(packet, &payload, &mut to_server)).unwrap();
                    read_until(&mut s, &mut from_server, ServerProt::IfOpentop);
                    // "Exit to Login" (182:13): the logout packet, then the server closes.
                    let (packet, payload) = client_packets::if_op(1, 182 << 16 | 13, -1, -1);
                    s.write_all(&client_prot::frame(packet, &payload, &mut to_server)).unwrap();
                    read_until(&mut s, &mut from_server, ServerProt::LogoutFull);
                    assert_eq!(s.read(&mut [0u8; 1]).unwrap_or(0), 0);
                })
            })
            .collect();
        for player in players {
            player.join().unwrap();
        }
        wait_until_empty(&service);
    }
}
