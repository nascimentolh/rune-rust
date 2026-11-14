//! The game map's rules both sides need: which tiles can be walked (`collision`) and the way
//! between two tiles (`route`). Ported from the reference client, which keeps them for its own
//! routes; the server uses them for its players'. Neither the client nor the server: built from
//! the cache's formats (`crate::cache`).

pub mod collision;
pub mod route;
