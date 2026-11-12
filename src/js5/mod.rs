//! The cache (JS5) service: handshake, then streams archives out of the cache on request.
//!
//! - `protocol`       — bytes on the wire
//! - `checksum_table` — the 255/255 table built at start-up
//! - here             — one client's session

pub mod checksum_table;
pub mod protocol;

use crate::cache::FileStore;
use crate::config::Settings;
use crate::net::{is_hang_up, Ctx};
use protocol::{Request, REQUEST_LEN};
use std::io::{Read, Write};
use std::sync::Mutex;

pub struct Js5 {
    // ponytail: one lock around the cache files; give each client its own file handles if many clients load at once.
    store: Mutex<FileStore>,
    checksum_table: Vec<u8>,
    build: i32,
    check_build: bool,
    grab_keys: Vec<i32>,
}

impl Js5 {
    pub fn new(store: FileStore, checksum_table: Vec<u8>, settings: &Settings) -> Self {
        Js5 {
            store: Mutex::new(store),
            checksum_table,
            build: settings.build,
            check_build: settings.check_build,
            grab_keys: settings.grab_keys.clone(),
        }
    }

    /// Runs after the service byte (15) has been read.
    pub fn session<S: Read + Write>(&self, stream: &mut S, ctx: &Ctx) -> std::io::Result<()> {
        let hs = protocol::read_handshake(stream)?;
        if self.check_build && hs.build != self.build {
            ctx.log(format!("cache: build {}, expected {} - sent 'out of date'", hs.build, self.build));
            return stream.write_all(&[protocol::STATUS_OUT_OF_DATE]);
        }
        ctx.log(format!("cache handshake: build {} sub-build {:?} key {:?}", hs.build, hs.sub_build, hs.key));
        stream.write_all(&protocol::encode_handshake_ok(&self.grab_keys))?;

        let mut xor_key = 0u8;
        let mut request = [0u8; REQUEST_LEN];
        loop {
            match stream.read_exact(&mut request) {
                Ok(()) => {}
                Err(e) if is_hang_up(&e) => return Ok(()),
                Err(e) => return Err(e),
            }
            match protocol::parse_request(request) {
                Request::Archive { index, archive, urgent } => self.send_archive(stream, ctx, index, archive, urgent, xor_key)?,
                Request::XorKey(key) => xor_key = key,
                Request::Close => return Ok(()),
                Request::LoggedIn | Request::LoggedOut | Request::Unknown(_) => {}
            }
        }
    }

    fn send_archive<S: Write>(&self, stream: &mut S, ctx: &Ctx, index: u8, archive: u32, urgent: bool, xor_key: u8) -> std::io::Result<()> {
        let container =
            if index == 255 && archive == 255 { Ok(self.checksum_table.clone()) } else { self.store.lock().unwrap().read(index, archive) };
        let reply = match container {
            Ok(c) => protocol::encode_response(index, archive, urgent, &c),
            Err(e) => {
                ctx.log(format!("asked for {index}/{archive}: {e}"));
                return Ok(());
            }
        };
        let Some(mut reply) = reply else {
            ctx.log(format!("asked for {index}/{archive}: malformed container"));
            return Ok(());
        };
        if xor_key != 0 {
            reply.iter_mut().for_each(|b| *b ^= xor_key);
        }
        stream.write_all(&reply)?;
        let mut s = ctx.stats.lock().unwrap();
        s.requests += 1;
        s.bytes_sent += reply.len() as u64;
        Ok(())
    }
}
