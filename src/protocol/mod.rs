//! The wire format both sides share: what the client sends and the server answers, byte for
//! byte as the build-727 reference client writes and reads it. Nothing here belongs to one side;
//! the client (`crate::client`) and the server (`crate::net`, `crate::js5`, `crate::login`,
//! `crate::players`) both build on it, and neither reaches into the other.
//!
//! - `js5`         — the cache service: handshake, file requests and replies
//! - `checksum_table` — the 255/255 table of every index's reference table
//! - `login`       — the login packet, its replies (world and lobby) and their codes
//! - `rsa`         — the login's secret block (and making keys)
//! - `isaac`       — the cipher both sides switch to after login
//! - `server_prot` — server → client game packets: ids, sizes, framing, variables
//! - `client_prot` — client → server game packets: ids, sizes, framing, `IF_OP`

pub mod checksum_table;
pub mod client_prot;
pub mod isaac;
pub mod js5;
pub mod login;
pub mod rsa;
pub mod server_prot;
