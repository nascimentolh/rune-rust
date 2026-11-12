//! Login wire format of the build-727 reference client (`jagex/Login.java`, `LoginProt.java`,
//! `ChatLine.getLoginMod`, `Class94.method1587`, `ByteBuf.applyRSA` / `encryptWithXtea`).
//!
//! The client sends service byte 14 alone (`Login.java:93`); the server answers 0 (`:104-117`).
//! Then (`Login.java:165-266`):
//!   type u8 (16 world, 19 lobby — the reference always uses 16, `Loader.LOBBY_ENABLED = false`) ·
//!   size u16 · build 727 i32 · minor 1 i32 · [world: reconnecting u8] ·
//!   RSA block: length u16 · bytes (`applyRSA`) ·
//!   XTEA block (the rest, key = the 4 seeds; only whole 8-byte blocks are encrypted)
//! RSA block once decrypted: 10 · 4 seeds i32 · 0 i64 · password string · i64 · i64
//! XTEA block: 1 if the name is text · username (text, or base-37 i64) · then
//!   world: window mode u8 · width u16 · height u16 · … (many more fields, not needed here)
//!   lobby: game u8 · language u8 · …
//! Strings end with a 0 byte.

use crate::cache::xtea;
use crate::config::Rsa;
use std::io::{self, Read};

/// `LoginProt.INIT_GAME_CONNECTION`, `GAMELOGIN`, `LOBBYLOGIN`.
pub const SERVICE_LOGIN: u8 = 14;
pub const TYPE_WORLD: u8 = 16;
pub const TYPE_LOBBY: u8 = 19;
pub const RSA_MAGIC: u8 = 10;

/// Reply codes.
pub const OK: u8 = 2;
pub const INVALID_LOGIN: u8 = 3;
/// The account is already in the game.
pub const ALREADY_ONLINE: u8 = 5;
pub const OUT_OF_DATE: u8 = 6;
/// The world has no free player index. Estimated: the usual code for "This world is full".
pub const WORLD_FULL: u8 = 7;
pub const BAD_SESSION: u8 = 10;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    World,
    Lobby,
}

#[derive(Debug, PartialEq)]
pub struct LoginRequest {
    pub kind: Kind,
    pub build: i32,
    pub sub_build: i32,
    /// World logins: the client is reconnecting after a lost connection (`GAME_STATE == 10`).
    pub reconnecting: bool,
    pub seeds: [i32; 4],
    pub password: String,
    pub username: String,
    /// World logins: (window mode, width, height).
    pub display: Option<(u8, u16, u16)>,
}

/// Reads one login packet: (type, payload).
pub fn read_packet(r: &mut impl Read) -> io::Result<(u8, Vec<u8>)> {
    let mut head = [0u8; 3];
    r.read_exact(&mut head)?;
    let mut payload = vec![0u8; u16::from_be_bytes([head[1], head[2]]) as usize];
    r.read_exact(&mut payload)?;
    Ok((head[0], payload))
}

/// A cursor over bytes; every read says what was missing when the data runs out.
struct Bytes<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Bytes<'a> {
    fn take(&mut self, n: usize, what: &str) -> Result<&'a [u8], String> {
        let out = self.data.get(self.pos..self.pos + n).ok_or(format!("packet ended before {what} (offset {})", self.pos))?;
        self.pos += n;
        Ok(out)
    }
    fn u8(&mut self, what: &str) -> Result<u8, String> {
        Ok(self.take(1, what)?[0])
    }
    fn u16(&mut self, what: &str) -> Result<u16, String> {
        Ok(u16::from_be_bytes(self.take(2, what)?.try_into().unwrap()))
    }
    fn i32(&mut self, what: &str) -> Result<i32, String> {
        Ok(i32::from_be_bytes(self.take(4, what)?.try_into().unwrap()))
    }
    fn i64(&mut self, what: &str) -> Result<i64, String> {
        Ok(i64::from_be_bytes(self.take(8, what)?.try_into().unwrap()))
    }
    /// 0-terminated, one byte per character (Latin-1).
    fn string(&mut self, what: &str) -> Result<String, String> {
        let rest = &self.data[self.pos..];
        let end = rest.iter().position(|&b| b == 0).ok_or(format!("{what} has no end marker"))?;
        self.pos += end + 1;
        Ok(rest[..end].iter().map(|&b| b as char).collect())
    }
}

/// Usernames sent as numbers: base 37 over "_a-z0-9".
pub fn base37_to_name(mut v: i64) -> String {
    const CHARS: &[u8] = b"_abcdefghijklmnopqrstuvwxyz0123456789";
    let mut out = Vec::new();
    while v > 0 {
        out.push(CHARS[(v % 37) as usize]);
        v /= 37;
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

/// Decodes a login packet. Without an RSA key the block is read as plain bytes.
pub fn decode(kind: u8, payload: &[u8], rsa: Option<&Rsa>) -> Result<LoginRequest, String> {
    let kind = match kind {
        TYPE_WORLD => Kind::World,
        TYPE_LOBBY => Kind::Lobby,
        other => return Err(format!("unknown login type {other}")),
    };
    let mut p = Bytes { data: payload, pos: 0 };
    let build = p.i32("build")?;
    let sub_build = p.i32("sub-build")?;
    let reconnecting = kind == Kind::World && p.u8("reconnecting flag")? == 1;
    let rsa_len = p.u16("RSA block size")? as usize;
    let rsa_raw = p.take(rsa_len, "RSA block")?;
    let rsa_plain = match rsa {
        Some(key) => super::rsa::crypt(rsa_raw, key),
        None => rsa_raw.to_vec(),
    };

    let mut r = Bytes { data: &rsa_plain, pos: 0 };
    let magic = r.u8("RSA magic")?;
    if magic != RSA_MAGIC {
        return Err(format!("RSA block starts with {magic}, expected {RSA_MAGIC} (wrong RSA key?)"));
    }
    let mut seeds = [0i32; 4];
    for s in &mut seeds {
        *s = r.i32("ISAAC seed")?;
    }
    r.i64("session check")?;
    let password = r.string("password")?;

    let mut xtea_block = payload[p.pos..].to_vec();
    xtea::decrypt(&mut xtea_block, &seeds);
    let mut x = Bytes { data: &xtea_block, pos: 0 };
    let username = if x.u8("name type")? == 1 { x.string("username")? } else { base37_to_name(x.i64("username")?) };
    let display = match kind {
        Kind::Lobby => None,
        Kind::World => Some((x.u8("window mode")?, x.u16("width")?, x.u16("height")?)),
    };
    Ok(LoginRequest { kind, build, sub_build, reconnecting, seeds, password, username, display })
}

/// Builds a login packet the way the reference client does, up to the window size; the
/// reference's later fields (preferences, machine info, cache CRCs …) aren't ported yet.
pub fn encode(req: &LoginRequest, public: Option<&Rsa>) -> Vec<u8> {
    let mut block = vec![RSA_MAGIC];
    req.seeds.iter().for_each(|s| block.extend(s.to_be_bytes()));
    block.extend(0i64.to_be_bytes());
    block.extend(req.password.bytes());
    block.push(0);
    block.extend([0u8; 16]);
    let rsa_block = match public {
        Some(key) => super::rsa::crypt(&block, key),
        None => block,
    };

    let mut tail = vec![1];
    tail.extend(req.username.bytes());
    tail.push(0);
    match req.display {
        Some((mode, w, h)) => {
            tail.push(mode);
            tail.extend(w.to_be_bytes());
            tail.extend(h.to_be_bytes());
        }
        // Game id and language.
        None => tail.extend([0, 0]),
    }
    // Like `encryptWithXtea`: whole 8-byte blocks only, the remainder stays plain.
    xtea::encrypt(&mut tail, &req.seeds);

    let mut payload = Vec::new();
    payload.extend(req.build.to_be_bytes());
    payload.extend(req.sub_build.to_be_bytes());
    if req.kind == Kind::World {
        payload.push(req.reconnecting as u8);
    }
    payload.extend((rsa_block.len() as u16).to_be_bytes());
    payload.extend(rsa_block);
    payload.extend(tail);

    let kind = match req.kind {
        Kind::World => TYPE_WORLD,
        Kind::Lobby => TYPE_LOBBY,
    };
    let mut out = vec![kind];
    out.extend((payload.len() as u16).to_be_bytes());
    out.extend(payload);
    out
}

/// Successful world-login reply, as `Login.java:377-403` reads it: code 2 · size u8 · rights ·
/// mod level · quick chat · verified email · (flag) · quick-chat only · player index u16 · member ·
/// date of birth u24 · members world · display name string.
pub fn encode_ok(rights: u8, player_index: u16, name: &str) -> Vec<u8> {
    let mut body = vec![rights, 0, 0, 0, 1, 0];
    body.extend(player_index.to_be_bytes());
    body.extend([1, 0, 0, 0, 1]);
    body.extend(name.bytes());
    body.push(0);
    let mut out = vec![OK, body.len() as u8];
    out.extend(body);
    out
}

/// Successful lobby-login reply, as `Login.java:409-450` reads it (lobby stage 140): code 2 ·
/// size u8 · rights · mod level · quick chat · date of birth u24 · gender · verified email ·
/// (flag) · membership end i64 · (5-byte) · member flags (1 member, 2 subscription) · coins i32 ·
/// loyalty u8 · loyalty balance i32 · recovery questions date u16 · unread messages u16 · last
/// login day u16 · last IP i32 · email status u8 · two u16 · (flag) · display name (GJ string:
/// a 0 byte, then the text) · u8 · i32 · (flag) · news world u16 (65535 none) · news host (GJ).
///
/// What this server says (its own choice, no reference server): everyone is a member until
/// `membership_end` (ms since 1970; the 5-byte field after it is the time left, which the
/// client counts down), verified, a player (rights 0), last seen from 127.0.0.1, no news server.
pub fn encode_lobby_ok(name: &str, membership_end: i64, now: i64) -> Vec<u8> {
    let mut body = vec![0, 0, 0]; // rights (player), mod level, quick chat
    body.extend([0, 0, 0]); // date of birth
    body.extend([0, 1, 0]); // gender, verified email, flag
    body.extend(membership_end.to_be_bytes());
    body.extend(&(membership_end - now).max(0).to_be_bytes()[3..]); // time left, 5 bytes
    body.push(1); // member
    body.extend(0i32.to_be_bytes()); // coins
    body.push(0);
    body.extend(0i32.to_be_bytes()); // loyalty balance
    body.extend([0u8; 6]); // recovery date, messages, last login day
    body.extend(0x7f00_0001i32.to_be_bytes()); // last IP
    body.push(0); // email status
    body.extend([0u8; 4]);
    body.push(0);
    body.push(0);
    body.extend(name.bytes());
    body.push(0);
    body.push(0);
    body.extend(0i32.to_be_bytes());
    body.push(0);
    body.extend(u16::MAX.to_be_bytes()); // no news world
    body.extend([0, 0]); // news host: GJ string ""
    let mut out = vec![OK, body.len() as u8];
    out.extend(body);
    out
}

/// The lobby reply (`Login.java:409-450`), the fields this client reads back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LobbyDetails {
    /// `PLAYER_RIGHTS`, `PLAYER_MOD_LEVEL`.
    pub rights: u8,
    pub mod_level: u8,
    /// `IS_MEMBER` (flags bit 0) and `IS_MEMBER_SUBSCRIPTION` (bit 1).
    pub member: bool,
    pub subscription: bool,
    /// `MEMBERSHIP_END` (ms since 1970) and the 5-byte value after it; the reference keeps
    /// `aLong420` = end − now − that value.
    pub membership_end: i64,
    pub membership_left: i64,
    /// `RECOVERYQUESTIONSSETDATE`, `MESSAGES`, `LASTLOGINLDAY` (days).
    pub recovery_day: u16,
    pub unread_messages: u16,
    pub last_login_day: u16,
    /// `LAST_IP_ADDRESS`.
    pub last_ip: i32,
    /// `EMAIL_STATUS`.
    pub email_status: u8,
    /// `anInt8307` (credit card expiry), `anInt5357` (grace period expiry).
    pub cc_expiry: u16,
    pub grace_expiry: u16,
    /// `displayName` (a GJ string: a 0 byte, then the text).
    pub name: String,
}

/// Reads a lobby reply body (after code and size), in `Login.java:409-450`'s order.
pub fn decode_lobby_ok(body: &[u8]) -> Result<LobbyDetails, String> {
    let mut b = Bytes { data: body, pos: 0 };
    let rights = b.u8("rights")?;
    let mod_level = b.u8("mod level")?;
    b.u8("quick chat")?;
    b.take(3, "date of birth")?;
    b.take(3, "gender, verified email, flag")?;
    let membership_end = b.i64("membership end")?;
    let five = b.take(5, "5-byte value")?;
    let membership_left = five.iter().fold(0i64, |v, &x| v << 8 | x as i64);
    let flags = b.u8("member flags")?;
    b.take(4 + 1 + 4, "coins, loyalty, loyalty balance")?;
    let recovery_day = b.u16("recovery questions date")?;
    let unread_messages = b.u16("unread messages")?;
    let last_login_day = b.u16("last login day")?;
    let last_ip = b.i32("last IP")?;
    let email_status = b.u8("email status")?;
    let cc_expiry = b.u16("card expiry")?;
    let grace_expiry = b.u16("grace expiry")?;
    b.u8("flag")?;
    if b.u8("GJ string marker")? != 0 {
        return Err("display name isn't a GJ string".into());
    }
    let name = b.string("display name")?;
    Ok(LobbyDetails {
        rights,
        mod_level,
        member: flags & 1 != 0,
        subscription: flags & 2 != 0,
        membership_end,
        membership_left,
        recovery_day,
        unread_messages,
        last_login_day,
        last_ip,
        email_status,
        cc_expiry,
        grace_expiry,
        name,
    })
}

/// Reads the body of a successful reply (after code and size): (player index, name).
pub fn decode_ok(body: &[u8]) -> Option<(u16, String)> {
    let index = u16::from_be_bytes(body.get(6..8)?.try_into().ok()?);
    let name = body.get(13..)?.iter().take_while(|&&b| b != 0).map(|&b| b as char).collect();
    Some((index, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(kind: Kind) -> LoginRequest {
        LoginRequest {
            kind,
            build: 727,
            sub_build: 1,
            reconnecting: false,
            seeds: [5, -6, 7, -8],
            password: "secret".into(),
            username: "tester".into(),
            display: (kind != Kind::Lobby).then_some((2, 765, 503)),
        }
    }

    #[test]
    fn login_round_trips_with_and_without_rsa() {
        let (public, private) = super::super::rsa::generate(512);
        for kind in [Kind::World, Kind::Lobby] {
            let req = request(kind);
            let wire = encode(&req, Some(&public));
            let (t, payload) = read_packet(&mut &wire[..]).unwrap();
            assert_eq!(decode(t, &payload, Some(&private)).unwrap(), req);
            let plain = encode(&req, None);
            let (t, payload) = read_packet(&mut &plain[..]).unwrap();
            assert_eq!(decode(t, &payload, None).unwrap(), req);
        }
    }

    #[test]
    fn ok_reply_round_trips() {
        let lobby = encode_lobby_ok("bob", 1_000_000, 400_000);
        let details = decode_lobby_ok(&lobby[2..]).unwrap();
        assert_eq!((details.member, details.name.as_str(), details.last_ip), (true, "bob", 0x7f00_0001));
        let wire = encode_ok(0, 42, "bob");
        assert_eq!(wire[0], OK);
        assert_eq!(decode_ok(&wire[2..]), Some((42, "bob".to_string())));
    }

    #[test]
    fn base37_names() {
        assert_eq!(base37_to_name(0), "");
        assert_eq!(base37_to_name(2 * 37 + 1), "ba");
    }
}
