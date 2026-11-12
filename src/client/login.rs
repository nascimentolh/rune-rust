//! Sending a login to the server in a background thread and reading its answer.

use super::lobby::LobbyConnection;
use crate::config::Rsa;
use crate::protocol::login::{self as protocol, Kind, LoginRequest};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Game window size sent with a world login.
const DISPLAY: (u8, u16, u16) = (2, 765, 503);

pub enum Outcome {
    Accepted {
        player_index: u16,
        name: String,
        /// Where the first game packet (`REGION`) put this player, and how many map regions
        /// it sent keys for.
        arrival: Arrival,
    },
    /// `code` is what the login window's scripts read (`Class9.anInt72`): the server's reply
    /// code, or -4 when the connection failed (`Login.java:568`).
    Refused { code: i32, why: String },
    /// A lobby login went through (`Login.java:409-450`); its connection stays open.
    Lobby(protocol::LobbyDetails, LobbyConnection),
}

/// `Login.java:568`: the connection failed.
pub const CONNECTION_FAILED: i32 = -4;
/// `Login.java:45`: no answer in time, three times over.
pub const TIMED_OUT: i32 = -5;
/// The server's "try again" reply (`Login.java:297`).
const TRY_AGAIN: i32 = 23;
/// `Class9.anInt104`: how many times a login is tried again after the first.
const RETRIES: u32 = 3;
/// How long one try may take: 250 client ticks the first time, 2000 after (`Login.java:30-34`).
const FIRST_TRY: Duration = Duration::from_millis(250 * 20);
const LATER_TRY: Duration = Duration::from_millis(2000 * 20);

pub type Pending = Arc<Mutex<Option<Outcome>>>;

/// What the first game packet said (`Class197.method3203`, `MapRegion.loadMapSceneNormal`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arrival {
    /// x, y, plane.
    pub at: (i32, i32, i32),
    /// The map's centre chunk.
    pub chunk: (i32, i32),
    pub regions: usize,
}

/// Reads the first game packet of a world login (`Login.java` stages 167, 173, 156): an
/// ISAAC-scrambled smart opcode (`readEncryptedSmart`), a u16 size, then `REGION`'s payload.
fn read_region<R: Read>(s: &mut R, cipher: &mut crate::protocol::isaac::Isaac) -> std::io::Result<Arrival> {
    let bad = |why: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, why.to_string());
    let mut b = [0u8; 1];
    s.read_exact(&mut b)?;
    let mut opcode = b[0].wrapping_sub(cipher.next() as u8) as u32;
    if opcode >= 128 {
        s.read_exact(&mut b)?;
        opcode = ((opcode - 128) << 8) + b[0].wrapping_sub(cipher.next() as u8) as u32;
    }
    if opcode != crate::protocol::server_prot::REGION {
        return Err(bad(&format!("first game packet was {opcode}, not REGION")));
    }
    let mut size = [0u8; 2];
    s.read_exact(&mut size)?;
    let mut body = vec![0u8; u16::from_be_bytes(size) as usize];
    s.read_exact(&mut body)?;
    // 30 bits of this player, then 18 per other index: 30 + 2046 × 18 bits, byte-aligned.
    let me = u32::from_be_bytes(body.get(..4).ok_or_else(|| bad("REGION too short"))?.try_into().unwrap_or_default()) >> 2;
    let at = ((me >> 14 & 0x3fff) as i32, (me & 0x3fff) as i32, (me >> 28) as i32);
    let map = body.get((30 + 2046 * 18_usize).div_ceil(8)..).ok_or_else(|| bad("REGION too short"))?;
    if map.len() < 6 {
        return Err(bad("REGION map part too short"));
    }
    let chunk = (u16::from_be_bytes([map[1], map[2]]) as i32, u16::from_be_bytes([map[3], map[4]]) as i32);
    Ok(Arrival { at, chunk, regions: (map.len() - 6) / 16 })
}

pub struct Target {
    pub address: String,
    pub build: i32,
    pub sub_build: i32,
    /// The server's public login key; `None` = send the secret part unencrypted.
    pub public_key: Option<Rsa>,
    /// Log in to the lobby (type 19) rather than the world (`MaterialProp35.lobbyLogin`).
    pub lobby: bool,
}

pub fn start(target: &Target, username: &str, password: &str) -> Pending {
    let pending: Pending = Arc::default();
    let out = Arc::clone(&pending);
    let request = LoginRequest {
        kind: if target.lobby { Kind::Lobby } else { Kind::World },
        build: target.build,
        sub_build: target.sub_build,
        reconnecting: false,
        seeds: [rand::random(), rand::random(), rand::random(), rand::random()],
        password: password.to_string(),
        username: username.to_string(),
        display: (!target.lobby).then_some(DISPLAY),
    };
    let (address, key) = (target.address.clone(), target.public_key.clone());
    thread::spawn(move || {
        *out.lock().unwrap() = Some(attempt(&address, &request, key.as_ref()));
    });
    pending
}

/// `Login.method5018`'s retries: a failed connection, a try that takes too long, and the
/// server's reply 23 each start over, up to three times; then -4 (failed) or -5 (too slow).
/// The reference also swaps to its other port on a retry; there is one address here.
fn attempt(address: &str, request: &LoginRequest, key: Option<&Rsa>) -> Outcome {
    let mut retries = 0;
    loop {
        let limit = if retries == 0 { FIRST_TRY } else { LATER_TRY };
        match send(address, request, key, limit) {
            Ok(Outcome::Refused { code: TRY_AGAIN, .. }) if retries < RETRIES => {}
            Ok(outcome) => return outcome,
            Err(_) if retries < RETRIES => {}
            Err(e) => {
                let slow = matches!(e.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock);
                let code = if slow { TIMED_OUT } else { CONNECTION_FAILED };
                return Outcome::Refused { code, why: format!("Connection problem: {e}") };
            }
        }
        retries += 1;
    }
}

/// What each reply code means to the player.
fn explain(code: u8) -> String {
    match code {
        protocol::INVALID_LOGIN => "Invalid username or password.".into(),
        protocol::ALREADY_ONLINE => "Your account is already logged in.".into(),
        protocol::OUT_OF_DATE => "The game has been updated. Please reload.".into(),
        protocol::BAD_SESSION => "The server couldn't read the login. Check the RSA key settings.".into(),
        other => format!("Login refused (code {other})."),
    }
}

fn send(address: &str, request: &LoginRequest, key: Option<&Rsa>, limit: Duration) -> std::io::Result<Outcome> {
    let to = address.to_socket_addrs()?.next().ok_or_else(|| std::io::Error::other("no address"))?;
    let mut s = TcpStream::connect_timeout(&to, limit)?;
    s.set_read_timeout(Some(limit))?;
    s.set_write_timeout(Some(limit))?;
    s.write_all(&[protocol::SERVICE_LOGIN])?;
    let mut status = [0u8; 1];
    s.read_exact(&mut status)?;
    if status[0] != 0 {
        return Ok(Outcome::Refused { code: status[0] as i32, why: explain(status[0]) });
    }
    s.write_all(&protocol::encode(request, key))?;
    let mut code = [0u8; 1];
    s.read_exact(&mut code)?;
    if code[0] != protocol::OK {
        return Ok(Outcome::Refused { code: code[0] as i32, why: explain(code[0]) });
    }
    let mut size = [0u8; 1];
    s.read_exact(&mut size)?;
    let mut body = vec![0u8; size[0] as usize];
    s.read_exact(&mut body)?;
    // A lobby login ends here (`Login.java:477`: login stage 2, game state 0).
    if request.kind == Kind::Lobby {
        let details = protocol::decode_lobby_ok(&body).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        return Ok(Outcome::Lobby(details, LobbyConnection::new(s, request.seeds)?));
    }
    let reply = protocol::decode_ok(&body).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "login reply too short"))?;
    let (_, mut from_server) = crate::protocol::isaac::Isaac::pair(request.seeds);
    let arrival = read_region(&mut s, &mut from_server)?;
    Ok(Outcome::Accepted { player_index: reply.0, name: reply.1, arrival })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn request() -> LoginRequest {
        LoginRequest {
            kind: Kind::World,
            build: 727,
            sub_build: 1,
            reconnecting: false,
            seeds: [0; 4],
            password: "p".into(),
            username: "u".into(),
            display: None,
        }
    }

    #[test]
    fn tries_three_more_times_then_gives_up() {
        // A server that takes each connection and closes it at once: every try fails.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let tries = Arc::new(AtomicU32::new(0));
        let counted = Arc::clone(&tries);
        thread::spawn(move || {
            for stream in listener.incoming() {
                counted.fetch_add(1, Ordering::SeqCst);
                drop(stream);
            }
        });
        let outcome = attempt(&address, &request(), None);
        assert!(matches!(outcome, Outcome::Refused { code: CONNECTION_FAILED, .. }));
        assert_eq!(tries.load(Ordering::SeqCst), 1 + RETRIES);
    }
}
