//! The network front door: a port per world, one thread per connection. The first byte a
//! client sends picks the service — the cache (JS5) or the login, which goes on to the lobby or
//! to the world whose port the client came in on. The first world's port is the server's own
//! (`server.cfg`), where the cache and the lobby are also reached.
//!
//! Also here: the counters and recent log lines every part of the server reports to (`Stats`),
//! and the handle a part reports through (`Ctx`).

use super::flags::Flag;
use super::js5::Js5;
use super::login::LoginService;
use super::trace::{TraceFile, Traced};
use super::worlds::{Hosted, Options, Worlds};
use std::collections::{BTreeMap, VecDeque};
use std::io::Read;
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const LOG_LINES: usize = 200;

/// A client's connection as the services get it.
pub type Connection = Traced<TcpStream>;

/// A game world's own numbers, written by its thread once a tick.
#[derive(Default)]
pub struct WorldStats {
    pub players: usize,
    pub ticks: u64,
    /// How long the last tick's work took, and the longest any has taken.
    pub last_tick: Duration,
    pub slowest_tick: Duration,
}

/// Counters and recent log lines, shown on the status screen and console.
#[derive(Default)]
pub struct Stats {
    pub connections: u64,
    pub active: u64,
    pub requests: u64,
    pub bytes_sent: u64,
    pub logins: u64,
    /// By world number.
    pub worlds: BTreeMap<i32, WorldStats>,
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

/// The one `Stats`, shared by every thread of the server.
pub type SharedStats = Arc<Mutex<Stats>>;

/// What a part of the server reports with while it deals with one client.
#[derive(Clone)]
pub struct Ctx {
    pub peer: String,
    pub stats: SharedStats,
}

impl Ctx {
    pub fn log(&self, line: impl AsRef<str>) {
        self.stats.lock().unwrap().log(format!("{} {}", self.peer, line.as_ref()));
    }
}

pub struct Server {
    js5: Js5,
    login: LoginService,
    trace: Option<TraceFile>,
    pub stats: SharedStats,
    pub worlds: Arc<Worlds>,
    /// The country flags the panel offers, read from the cache when the server started.
    pub flags: Vec<Flag>,
    /// The address every world's port is opened on, and the first world's port.
    ip: IpAddr,
    port: u16,
}

/// Ports tried after the server's own when the administrator leaves a world's port to the server.
const PORTS_TRIED: u16 = 1000;

impl Server {
    /// A server whose first world is `first`, reached on `listener`: starts accepting at once.
    pub fn start(
        js5: Js5,
        login: LoginService,
        trace: Option<TraceFile>,
        worlds: Arc<Worlds>,
        flags: Vec<Flag>,
        listener: TcpListener,
        first: Options,
    ) -> std::io::Result<Arc<Self>> {
        let address = listener.local_addr()?;
        let stats = Arc::clone(&login.stats);
        let server = Arc::new(Server { js5, login, trace, stats, worlds, flags, ip: address.ip(), port: address.port() });
        let world = server.worlds.open(Options { port: address.port(), ..first });
        server.listen(listener, world);
        Ok(server)
    }

    /// Opens another world on its own port, as the administrator asked.
    pub fn open_world(self: &Arc<Self>, options: Options) -> Result<Arc<Hosted>, String> {
        options.check()?;
        let listener = TcpListener::bind((self.ip, options.port)).map_err(|e| format!("port {} can't be used: {e}", options.port))?;
        let world = self.worlds.open(options);
        self.listen(listener, Arc::clone(&world));
        Ok(world)
    }

    /// The first port after the server's own that nothing is listening on.
    pub fn free_port(&self) -> Option<u16> {
        (1..=PORTS_TRIED).filter_map(|n| self.port.checked_add(n)).find(|&port| TcpListener::bind((self.ip, port)).is_ok())
    }

    /// Accepts `world`'s clients in a background thread.
    fn listen(self: &Arc<Self>, listener: TcpListener, world: Arc<Hosted>) {
        if let Ok(addr) = listener.local_addr() {
            self.stats.lock().unwrap().log(format!("World {} listening on {addr}", world.number));
        }
        let server = Arc::clone(self);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let (server, world) = (Arc::clone(&server), Arc::clone(&world));
                thread::spawn(move || server.serve_client(stream, &world));
            }
        });
    }

    fn serve_client(&self, stream: TcpStream, world: &Hosted) {
        let peer = stream.peer_addr().map_or("?".into(), |a| a.to_string());
        let _ = stream.set_nodelay(true);
        let ctx = Ctx { peer, stats: Arc::clone(&self.stats) };
        {
            let mut s = self.stats.lock().unwrap();
            s.connections += 1;
            s.active += 1;
        }
        ctx.log("connected");
        let mut stream = Traced::new(stream, self.trace.clone(), &ctx.peer);
        let result = self.dispatch(&mut stream, &ctx, world);
        self.stats.lock().unwrap().active -= 1;
        ctx.log(match result {
            Ok(()) => "disconnected".to_string(),
            Err(e) if is_hang_up(&e) => "disconnected".to_string(),
            Err(e) => format!("disconnected: {e}"),
        });
    }

    fn dispatch(&self, stream: &mut Connection, ctx: &Ctx, world: &Hosted) -> std::io::Result<()> {
        let mut service = [0u8; 1];
        stream.read_exact(&mut service)?;
        match service[0] {
            crate::protocol::js5::SERVICE_JS5 => self.js5.session(stream, ctx),
            crate::protocol::login::SERVICE_LOGIN => self.login.session(stream, ctx, world),
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
