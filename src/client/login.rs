//! Sending a login to the server in a background thread and reading its answer.

use crate::config::Rsa;
use crate::login::protocol::{self, Kind, LoginRequest};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Game window size sent with a world login.
const DISPLAY: (u8, u16, u16) = (2, 765, 503);

#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Accepted { player_index: u16, name: String },
    Refused(String),
}

pub type Pending = Arc<Mutex<Option<Outcome>>>;

pub struct Target {
    pub address: String,
    pub build: i32,
    pub sub_build: i32,
    /// The server's public login key; `None` = send the secret part unencrypted.
    pub public_key: Option<Rsa>,
}

pub fn start(target: &Target, username: &str, password: &str) -> Pending {
    let pending: Pending = Arc::default();
    let out = Arc::clone(&pending);
    let request = LoginRequest {
        kind: Kind::World,
        build: target.build,
        sub_build: target.sub_build,
        reconnecting: false,
        seeds: [rand::random(), rand::random(), rand::random(), rand::random()],
        password: password.to_string(),
        username: username.to_string(),
        display: Some(DISPLAY),
    };
    let (address, key) = (target.address.clone(), target.public_key.clone());
    thread::spawn(move || {
        let outcome = send(&address, &request, key.as_ref()).unwrap_or_else(|e| Outcome::Refused(format!("Connection problem: {e}")));
        *out.lock().unwrap() = Some(outcome);
    });
    pending
}

/// What each reply code means to the player.
fn explain(code: u8) -> String {
    match code {
        protocol::INVALID_LOGIN => "Invalid username or password.".into(),
        protocol::OUT_OF_DATE => "The game has been updated. Please reload.".into(),
        protocol::BAD_SESSION => "The server couldn't read the login. Check the RSA key settings.".into(),
        other => format!("Login refused (code {other})."),
    }
}

fn send(address: &str, request: &LoginRequest, key: Option<&Rsa>) -> std::io::Result<Outcome> {
    let mut s = TcpStream::connect(address)?;
    s.set_read_timeout(Some(Duration::from_secs(15)))?;
    s.write_all(&[protocol::SERVICE_LOGIN])?;
    let mut status = [0u8; 1];
    s.read_exact(&mut status)?;
    if status[0] != 0 {
        return Ok(Outcome::Refused(explain(status[0])));
    }
    s.write_all(&protocol::encode(request, key))?;
    let mut code = [0u8; 1];
    s.read_exact(&mut code)?;
    if code[0] != protocol::OK {
        return Ok(Outcome::Refused(explain(code[0])));
    }
    let mut size = [0u8; 1];
    s.read_exact(&mut size)?;
    let mut body = vec![0u8; size[0] as usize];
    s.read_exact(&mut body)?;
    let reply = protocol::decode_ok(&body).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "login reply too short"))?;
    Ok(Outcome::Accepted { player_index: reply.0, name: reply.1 })
}
