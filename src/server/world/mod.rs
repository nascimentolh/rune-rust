//! A game world: one thread owns every player in it and runs the game tick.
//!
//! - `seats`      — the player indices (the only state other threads touch)
//! - `connection` — a player's two network threads and the queues between them and the world
//! - `player`     — what the world keeps for a player, and the packets queued for them
//! - `enter`      — what a player is sent on entering
//! - `interfaces` — the game's window and the panels opened in it
//! - `handlers`   — one function per client packet the world understands
//! - here         — the world's thread: players entering, the tick, players leaving
//!
//! A server runs as many worlds as its administrator opens (`super::worlds`). Each is one of
//! these, with its own thread, seats and players; two worlds share nothing.
//!
//! The rule that keeps this simple with hundreds of players: **only the world's thread reads
//! or changes game state**, so nothing here is locked. Network threads never touch a `Player`;
//! they hand packets over through queues. A client packet waits in its player's queue until
//! the next tick, the tick runs every player's packets in index order, and everything queued
//! for a player goes out as one write at the end of the tick.
//!
//! The tick, in order (`Game::tick`; new stages go where the comments say):
//! 1. read: each player's packets that have arrived, through `handlers`;
//! 2. flush: each player's queued packets, framed and handed to their writer;
//! 3. leave: players who logged out, hung up or can't keep up are let go.
//!
//! A player entering is dealt with as soon as the login hands them over, between ticks, so
//! logging in doesn't wait for one.

mod connection;
mod enter;
mod handlers;
mod interfaces;
mod player;
mod seats;

pub use enter::WELCOME;
pub use seats::{Seat, Seats, MAX_INDEX as MAX_PLAYERS};

use super::login::Online;
use super::net::{Connection, Ctx, SharedStats};
use super::session;
use crate::cache::xtea::Key;
use crate::protocol::isaac::Isaac;
use connection::Link;
use player::Player;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The game tick. This server's choice (there is no reference server): the 600 ms cycle these
/// game servers are known to run on.
pub const TICK: Duration = Duration::from_millis(600);

/// A player on the way in, as the login hands them over.
pub struct Arrival {
    pub name: String,
    /// The login's window mode (1 fixed, 2 resizable, 3 full screen), when it sent one.
    pub window_mode: Option<u8>,
    pub seat: Seat,
    pub online: Online,
}

/// A player in the world: the game's side, the network's side, and what holding them keeps
/// taken (the index and the name) until the world lets them go.
struct Session {
    player: Player,
    link: Link,
    _seat: Seat,
    _online: Online,
}

/// The handle on a running world, for the threads outside it.
pub struct World {
    seats: Arc<Seats>,
    arrivals: Sender<Box<Session>>,
}

impl World {
    /// Starts world `number` for at most `capacity` players. Its thread runs until the handle
    /// is dropped.
    pub fn start(number: i32, capacity: u16, map_keys: Arc<HashMap<u32, Key>>, stats: SharedStats) -> Self {
        let seats = Seats::new(capacity);
        let (arrivals, incoming) = channel();
        let game = Game { number, sessions: (0..=seats::MAX_INDEX).map(|_| None).collect(), map_keys, stats, ticks: 0 };
        std::thread::spawn(move || game.run(incoming));
        World { seats, arrivals }
    }

    pub fn seats(&self) -> &Arc<Seats> {
        &self.seats
    }

    /// Puts a logged-in player in the world and stays on their connection, reading their
    /// packets for the world, until it ends. Called on the connection's own thread.
    pub fn play(
        &self,
        stream: &mut Connection,
        ctx: &Ctx,
        arrival: Arrival,
        (from_client, to_client): (Isaac, Isaac),
    ) -> std::io::Result<()> {
        let (link, feed) = connection::open(stream, ctx.clone(), to_client)?;
        let player = Player::new(arrival.seat.index, arrival.name, arrival.window_mode);
        let session = Session { player, link, _seat: arrival.seat, _online: arrival.online };
        self.arrivals.send(Box::new(session)).map_err(|_| std::io::Error::other("the world has stopped"))?;
        connection::read(stream, from_client, feed)
    }
}

/// Everything the world's thread owns.
struct Game {
    /// This world's number, for its numbers on the status screen and its log lines.
    number: i32,
    /// The players, by index (`sessions[0]` is never used: the client has no player 0).
    sessions: Vec<Option<Session>>,
    /// Map keys by region, for `REGION` (the same for every world).
    map_keys: Arc<HashMap<u32, Key>>,
    stats: SharedStats,
    ticks: u64,
}

impl Game {
    fn run(mut self, arrivals: Receiver<Box<Session>>) {
        let mut next_tick = Instant::now() + TICK;
        loop {
            match arrivals.recv_timeout(next_tick.saturating_duration_since(Instant::now())) {
                Ok(session) => self.enter(*session),
                Err(RecvTimeoutError::Timeout) => {
                    let started = Instant::now();
                    self.tick();
                    self.report(started.elapsed());
                    // Keep the rhythm; after a long stall start again from now rather than
                    // running the missed ticks in a burst.
                    next_tick += TICK;
                    if next_tick < Instant::now() {
                        next_tick = Instant::now() + TICK;
                    }
                }
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    }

    /// A player comes in: they are sent the world at once.
    fn enter(&mut self, mut session: Session) {
        enter::first_packets(&mut session.player, &self.map_keys);
        let index = usize::from(session.player.index);
        if session.link.send(session.player.take_queued()) {
            self.sessions[index] = Some(session);
        }
    }

    fn tick(&mut self) {
        // 1. Read: what each player sent since the last tick.
        for session in self.sessions.iter_mut().flatten() {
            let Some(packets) = session.link.receive() else {
                session.player.leaving = true;
                continue;
            };
            for (packet, payload) in packets {
                session.link.ctx.log(session::describe(packet, &payload));
                handlers::handle(&mut session.player, packet, &payload);
            }
        }
        // Stages that move the world on (movement, then what each player sees of the others)
        // go here, between reading and flushing, each a loop over the players in index order.

        // 2. Flush, and 3. let go of who is leaving or can't be written to. A player who
        // can't be written to is told apart in the log: what was queued for them (a logout
        // packet, say) never reached them, so they are gone without being told why.
        for slot in &mut self.sessions {
            let Some(session) = slot else { continue };
            let sent = session.link.send(session.player.take_queued());
            if !sent {
                session.link.ctx.log(format!("'{}' fell too far behind to be written to - let go", session.player.name));
            } else if session.player.leaving {
                session.link.ctx.log(format!("'{}' left the world", session.player.name));
            }
            if session.player.leaving || !sent {
                *slot = None;
            }
        }
        self.ticks += 1;
    }

    /// The tick's numbers for the status screen, and a log line when one overran.
    fn report(&self, took: Duration) {
        let mut stats = self.stats.lock().unwrap();
        let world = stats.worlds.entry(self.number).or_default();
        world.players = self.sessions.iter().flatten().count();
        world.ticks = self.ticks;
        world.last_tick = took;
        world.slowest_tick = world.slowest_tick.max(took);
        if took > TICK {
            stats.log(format!("world {}: tick {} took {} ms, longer than a tick", self.number, self.ticks, took.as_millis()));
        }
    }
}
