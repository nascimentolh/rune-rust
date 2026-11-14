//! The wire format both sides share: what the client sends and the server answers, byte for
//! byte as the build-727 reference client writes and reads it. Nothing here belongs to one side;
//! the client (`crate::client`) and the server (`crate::server`) both build on it, and neither
//! reaches into the other. Every packet id and every payload layout lives here and nowhere else.
//!
//! - `js5`         — the cache service: handshake, file requests and replies
//! - `checksum_table` — the 255/255 table of every index's reference table
//! - `login`       — the login packet, its replies (world and lobby) and their codes
//! - `rsa`         — the login's secret block (and making keys)
//! - `isaac`       — the cipher both sides switch to after login
//! - `packet`      — what a packet table is made of (sizes, the table macro)
//! - `server_prot` — server → client game packets: the table (ids, names, sizes) and framing
//! - `server_packets` — the payload of each ported server packet
//! - `client_prot` — client → server game packets: the table and framing
//! - `client_packets` — the payload of each ported client packet, written and read back
//! - `huffman`     — the chat's text compression
//! - `region`      — the `REGION` packet
//! - `world_list`  — the lobby's world list packet
//! - `doc`         — writes `docs/PROTOCOL.md` from the tables

pub mod checksum_table;
pub mod client_packets;
pub mod client_prot;
pub mod doc;
pub mod huffman;
pub mod isaac;
pub mod js5;
pub mod login;
pub mod packet;
pub mod region;
pub mod rsa;
pub mod server_packets;
pub mod server_prot;
pub mod world_list;
