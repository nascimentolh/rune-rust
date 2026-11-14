//! What the world does with each client packet: one row here per packet, one function per
//! row, grouped in a file per area of the game. A packet with no row is read (and logged) and
//! nothing else: that is what "not ported" means on this side.
//!
//! A handler runs on the world's thread during the tick's read stage. It gets the player who
//! sent the packet and the payload, reads the payload with the decoder beside its encoder in
//! `crate::protocol::client_packets`, changes the player and queues packets for them. It
//! never writes to a socket, never blocks, and trusts nothing in the payload: a short or
//! senseless one is ignored.

mod chat;
mod interface;

use super::player::Player;
use crate::protocol::client_prot::ClientProt;
use crate::protocol::huffman::Huffman;

/// `huffman` is the cache's chat table, for the packets that carry text.
pub fn handle(player: &mut Player, packet: ClientProt, payload: &[u8], huffman: Option<&Huffman>) {
    match packet {
        ClientProt::ChatType => chat::chat_type(player, payload),
        ClientProt::Chat => chat::chat(player, payload, huffman),
        ClientProt::ScreenSize => interface::screen_size(player, payload),
        ClientProt::IfOp1 => interface::option_1(player, payload),
        _ => {}
    }
}
