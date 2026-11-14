//! The server. One process, a port per game world; how it is built and the rules for adding to it are in
//! `docs/SERVER.md` (read it before changing anything here).
//!
//! A connection's first byte picks a service, and each service is a folder or file here:
//!
//! - `app`      — starting the process: load the cache, build the services, listen
//! - `loader`, `screen`, `trace` — start-up steps, the status window, the byte-level log
//! - `net`      — the ports: one thread per connection, picks the service; counters and log
//! - `session`  — reading a logged-in client's packets (shared by the lobby and the world)
//! - `js5`      — the cache service
//! - `login`    — the login: checks, accounts, the reply, then hands over to the lobby or world
//! - `lobby`    — a lobby player: answered at once on its own connection thread
//! - `world`    — a game world: one thread owns every player in it and runs the game tick
//! - `worlds`   — the worlds this server runs, each on its own port
//! - `flags`    — the country flags of the world list, read from the cache for the panel
//! - `panel`    — the administrator's part of the status window: opening worlds
//!
//! The bytes on the wire are never built here: packets come from `crate::protocol`. Nothing
//! here uses `crate::client`, and the client uses nothing from here.

pub mod app;
mod flags;
mod js5;
pub mod loader;
mod lobby;
mod login;
pub mod net;
mod panel;
pub mod screen;
mod session;
mod trace;
pub mod world;
mod worlds;
