//! A world player's connection: the only place in `world` that touches a socket.
//!
//! Two threads per player, joined to the world's thread by two bounded queues:
//! - the reader (the connection's own thread) decodes client packets into the inbox. When the
//!   inbox is full it waits, so a client that floods is slowed by TCP itself and never grows
//!   the server's memory or one tick's work;
//! - the writer takes one buffer per tick from the outbox and writes it. The world never
//!   writes to a socket, so a client that stops reading can't hold up the tick: its outbox
//!   fills and the world lets the player go.
//!
//! Dropping the `Link` is how the world closes a connection: the writer writes what is left
//! (a logout packet, say), shuts the socket down, and the reader stops.

use crate::protocol::client_prot::Packet as ClientPacket;
use crate::protocol::isaac::Isaac;
use crate::protocol::server_prot::{self, Packet};
use crate::server::net::{Connection, Ctx};
use crate::server::session;
use std::io::Write;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};

/// Client packets a player can have waiting, and the most the world reads from them in one
/// tick. This server's choice: far more than a client sends in 600 ms.
const INBOX: usize = 64;
/// Ticks of output a client may fall behind before it is let go. This server's choice: about
/// five seconds.
const OUTBOX: usize = 8;
/// The writer only copies bytes to the socket, so it needs little stack; with two threads per
/// player this keeps a full world's memory small.
const WRITER_STACK: usize = 128 * 1024;

/// The world's end of a player's connection.
pub struct Link {
    inbox: Receiver<ClientPacket>,
    outbox: SyncSender<Vec<u8>>,
    /// The server's cipher for this client: packets must be framed in the order they are sent.
    to_client: Isaac,
    pub ctx: Ctx,
}

impl Link {
    /// The packets that have arrived (at most `INBOX`), or `None` once the client has gone and
    /// nothing is left to read.
    pub fn receive(&mut self) -> Option<Vec<ClientPacket>> {
        let mut packets = Vec::new();
        while packets.len() < INBOX {
            match self.inbox.try_recv() {
                Ok(packet) => packets.push(packet),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) if packets.is_empty() => return None,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        Some(packets)
    }

    /// Frames `packets` and hands them to the writer as one buffer. `false` when the client
    /// has gone or has fallen too far behind: the player must be let go (the cipher has moved
    /// on, so nothing more can be sent to them).
    pub fn send(&mut self, packets: Vec<Packet>) -> bool {
        if packets.is_empty() {
            return true;
        }
        let mut bytes = Vec::new();
        for (packet, payload) in packets {
            bytes.extend(server_prot::frame(packet, &payload, &mut self.to_client));
        }
        self.outbox.try_send(bytes).is_ok()
    }
}

/// Starts the writer for `stream` and returns the world's end of the connection with the
/// reader's end of the inbox.
pub fn open(stream: &Connection, ctx: Ctx, to_client: Isaac) -> std::io::Result<(Link, SyncSender<ClientPacket>)> {
    let (feed, inbox) = sync_channel(INBOX);
    let (outbox, pending) = sync_channel::<Vec<u8>>(OUTBOX);
    let mut writer = stream.try_clone()?;
    std::thread::Builder::new().name(format!("writer {}", ctx.peer)).stack_size(WRITER_STACK).spawn(move || {
        for bytes in pending {
            if writer.write_all(&bytes).is_err() {
                break;
            }
        }
        writer.shut_down();
    })?;
    Ok((Link { inbox, outbox, to_client, ctx }, feed))
}

/// The reader: decodes the client's packets into the inbox until the client hangs up or the
/// world has let the player go.
pub fn read(stream: &mut Connection, mut from_client: Isaac, feed: SyncSender<ClientPacket>) -> std::io::Result<()> {
    while let Some(packet) = session::read_packet(stream, &mut from_client)? {
        if feed.send(packet).is_err() {
            break;
        }
    }
    Ok(())
}
