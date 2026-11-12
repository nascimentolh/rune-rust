//! The network front door: one port, one thread per client. The first byte a
//! client sends picks the service — the cache (JS5) or the login server.

use crate::js5::{self, Js5};
use crate::login::{self, LoginService};
use crate::trace::{TraceFile, Traced};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

const LOG_LINES: usize = 200;

/// Counters and recent log lines, shown on the status screen and console.
#[derive(Default)]
pub struct Stats {
    pub connections: u64,
    pub active: u64,
    pub requests: u64,
    pub bytes_sent: u64,
    pub logins: u64,
    pub log: VecDeque<String>,
    /// Total lines ever logged (so the console knows what it hasn't printed yet).
    pub logged: u64,
}

impl Stats {
    pub fn log(&mut self, line: String) {
        self.log.push_back(line);
        self.logged += 1;
        if self.log.len() > LOG_LINES {
            self.log.pop_front();
        }
    }
}

/// What a service gets to report with while it talks to one client.
pub struct Ctx<'a> {
    pub peer: &'a str,
    pub stats: &'a Mutex<Stats>,
}

impl Ctx<'_> {
    pub fn log(&self, line: impl AsRef<str>) {
        self.stats.lock().unwrap().log(format!("{} {}", self.peer, line.as_ref()));
    }
}

pub struct Server {
    js5: Js5,
    login: LoginService,
    trace: Option<TraceFile>,
    pub stats: Mutex<Stats>,
}

impl Server {
    pub fn new(js5: Js5, login: LoginService, trace: Option<TraceFile>) -> Self {
        Server { js5, login, trace, stats: Mutex::default() }
    }

    /// Accepts clients in a background thread.
    pub fn start(self: &Arc<Self>, listener: TcpListener) {
        if let Ok(addr) = listener.local_addr() {
            self.stats.lock().unwrap().log(format!("Listening on {addr}"));
        }
        let server = Arc::clone(self);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let server = Arc::clone(&server);
                thread::spawn(move || server.serve_client(stream));
            }
        });
    }

    fn serve_client(&self, stream: TcpStream) {
        let peer = stream.peer_addr().map_or("?".into(), |a| a.to_string());
        let _ = stream.set_nodelay(true);
        let ctx = Ctx { peer: &peer, stats: &self.stats };
        {
            let mut s = self.stats.lock().unwrap();
            s.connections += 1;
            s.active += 1;
        }
        ctx.log("connected");
        let mut stream = Traced::new(stream, self.trace.clone(), &peer);
        let result = self.dispatch(&mut stream, &ctx);
        self.stats.lock().unwrap().active -= 1;
        ctx.log(match result {
            Ok(()) => "disconnected".to_string(),
            Err(e) if is_hang_up(&e) => "disconnected".to_string(),
            Err(e) => format!("disconnected: {e}"),
        });
    }

    fn dispatch<S: Read + Write>(&self, stream: &mut S, ctx: &Ctx) -> std::io::Result<()> {
        let mut service = [0u8; 1];
        stream.read_exact(&mut service)?;
        match service[0] {
            js5::protocol::SERVICE_JS5 => self.js5.session(stream, ctx),
            login::protocol::SERVICE_LOGIN => self.login.session(stream, ctx),
            other => {
                ctx.log(format!("asked for unknown service {other}"));
                Ok(())
            }
        }
    }
}

/// The client closing the connection, politely or not.
pub fn is_hang_up(e: &std::io::Error) -> bool {
    use std::io::ErrorKind::*;
    matches!(e.kind(), UnexpectedEof | ConnectionReset | ConnectionAborted)
}
