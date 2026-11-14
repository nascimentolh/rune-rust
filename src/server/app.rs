//! Running the server process: load the cache, start both services, and show
//! progress either in its status window or in a terminal (`--console`).

use super::flags;
use super::loader::{self, Loader};
use super::net::{Server, SharedStats};
use super::panel::Panel;
use super::worlds::{Options, Worlds};
use super::{js5, login, screen, trace};
use crate::config::Settings;
use macroquad::prelude::*;
use std::net::TcpListener;
use std::sync::Arc;

/// After loading: build the checksum table, set up the services, start the first world (as
/// `server.cfg` describes it) and start listening.
pub fn start_server(loader: &mut Loader, settings: &Settings, bind: &str) -> Result<(Arc<Server>, String), String> {
    let revisions = loader.revisions();
    let mut cache = loader.cache.take().ok_or("the cache didn't load")?;
    // The country flags come out of the cache before its files go to the JS5 service.
    let flags = flags::load(&mut cache);
    let mut store = cache.store;
    let checksums = js5::index_checksums(&mut store, &revisions);
    let table = crate::protocol::checksum_table::build(&checksums, settings.js5_rsa.as_ref());
    let trace = match &settings.trace_file {
        Some(path) => Some(trace::TraceFile::open(path).map_err(|e| format!("can't open trace {}: {e}", path.display()))?),
        None => None,
    };
    let listener = TcpListener::bind(bind).map_err(|e| format!("can't listen on {bind}: {e}"))?;
    let address = listener.local_addr().map_err(|e| e.to_string())?.to_string();
    let stats = SharedStats::default();
    let worlds = Worlds::new(loader.keys.by_region(), Arc::clone(&stats), settings.worlds_file.clone());
    let login = login::LoginService::new(settings, Arc::clone(&worlds), stats);
    // The first world: how many players `server.cfg` says, everything else as a world starts
    // out (a members world with no name, in the default place). The port is the listener's.
    let first = Options { capacity: settings.world_capacity, ..Options::default() };
    let server =
        Server::start(js5::Js5::new(store, table, settings), login, trace, worlds, flags, listener, first).map_err(|e| e.to_string())?;
    server.stats.lock().unwrap().log(format!("{} country flags in the cache", server.flags.len()));
    Ok((server, address))
}

/// Opens the worlds the administrator saved. Only the real server process does this: a
/// self-test's or a screenshot's server keeps to its one world.
fn open_saved_worlds(server: &Arc<Server>) {
    for options in server.worlds.saved() {
        if let Err(e) = server.open_world(options) {
            server.stats.lock().unwrap().log(format!("a saved world wasn't opened: {e}"));
        }
    }
}

/// Loads everything without drawing, then starts the server (used by tests and screenshots).
pub fn start_now(settings: &Settings, bind: &str) -> Result<(Arc<Server>, String, Loader), String> {
    let mut loader = Loader::new(settings.cache_dir.clone(), settings.keys_file.clone());
    while !loader.finished() {
        loader.step();
    }
    let (server, address) = start_server(&mut loader, settings, bind)?;
    Ok((server, address, loader))
}

/// `--gen-rsa`: prints a new key pair — the private half for server.cfg, the public half for the client.
pub fn print_new_rsa_keys() {
    const BITS: u64 = 1024;
    for name in ["login_rsa", "rsa"] {
        let (public, private) = crate::protocol::rsa::generate(BITS);
        println!("# {name}: put these two lines in server.cfg");
        println!("{name}_exponent = {}", private.exponent);
        println!("{name}_modulus = {}", private.modulus);
        println!("# and put this public key in the client: exponent {}, modulus", public.exponent);
        println!("# {}", public.modulus);
        println!();
    }
}

pub fn run_console(settings: Settings) {
    let mut loader = Loader::new(settings.cache_dir.clone(), settings.keys_file.clone());
    while !loader.finished() {
        loader.step();
    }
    println!("Cache: {}", settings.cache_dir.display());
    println!("{}", loader.status());
    let r = &loader.key_report;
    println!("Map keys: {} loaded, {} ok, {} wrong, {} missing {}", r.total, r.ok, r.wrong, r.missing, r.error.clone().unwrap_or_default());
    let (server, _) = match start_server(&mut loader, &settings, &format!("0.0.0.0:{}", settings.port)) {
        Ok(s) => s,
        Err(e) => {
            // Keep the terminal open so the reason can be read.
            eprintln!("{e}\nPress Enter to close.");
            let _ = std::io::stdin().read_line(&mut String::new());
            return;
        }
    };
    open_saved_worlds(&server);
    if let Some(path) = &settings.trace_file {
        println!("Byte trace: {}", path.display());
    }
    let mut printed = 0u64;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(250));
        let stats = server.stats.lock().unwrap();
        let unseen = (stats.logged - printed) as usize;
        for line in stats.log.iter().skip(stats.log.len().saturating_sub(unseen)) {
            println!("{line}");
        }
        printed = stats.logged;
    }
}

/// The server's status window: loading progress, then connections, log and the worlds.
pub async fn run_window(settings: Settings) {
    let bind = format!("0.0.0.0:{}", settings.port);
    let mut loader = Loader::new(settings.cache_dir.clone(), settings.keys_file.clone());
    let mut server: Option<(Arc<Server>, String)> = None;
    let mut panel = Panel::default();
    loop {
        if !loader.finished() {
            loader.step();
            if loader.finished() && loader.cache.is_some() {
                match start_server(&mut loader, &settings, &bind) {
                    Ok(s) => {
                        open_saved_worlds(&s.0);
                        server = Some(s);
                    }
                    Err(e) => loader.stage = loader::Stage::Failed(e),
                }
            }
        }
        if let Some((server, _)) = &server {
            panel.update(server);
        }
        screen::draw(&loader, server.as_ref().map(|(s, a)| (s.as_ref(), a.as_str())), &mut panel);
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::login::{self, Kind, LoginRequest};
    use std::io::{Read, Write};
    use std::net::TcpStream;

    /// A second world opens on its own port; a world login on that port lands in that world and
    /// not in the first; a taken port is refused; and the world is saved to be opened again.
    #[test]
    fn each_world_has_its_own_port_and_its_own_players() {
        let dir = std::env::temp_dir().join("rune-rust-worlds-test");
        let (cache_dir, keys_file) = crate::selftest::sample::write(&dir).unwrap();
        let worlds_file = dir.join("worlds.txt");
        let _ = std::fs::remove_file(&worlds_file);
        let settings = Settings { cache_dir, keys_file, accounts_file: None, worlds_file: Some(worlds_file), ..Settings::default() };
        let (server, _, _) = start_now(&settings, "127.0.0.1:0").unwrap();

        let port = server.free_port().unwrap();
        let options = Options { name: "Trade".into(), port, capacity: 5, lootshare: true, ..Options::default() };
        let second = server.open_world(options.clone()).unwrap();
        assert_eq!((second.number, server.worlds.all().len()), (2, 2));
        assert!(server.open_world(options.clone()).is_err());
        assert_ne!(server.free_port(), Some(port));

        let request = LoginRequest {
            kind: Kind::World,
            build: 727,
            sub_build: 1,
            reconnecting: false,
            seeds: [1, 2, 3, 4],
            password: "pw".into(),
            username: "bob".into(),
            display: Some((1, 765, 503)),
        };
        let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
        client.write_all(&[login::SERVICE_LOGIN]).unwrap();
        client.write_all(&login::encode(&request, None)).unwrap();
        let mut reply = [0u8; 2];
        client.read_exact(&mut reply).unwrap();
        assert_eq!(reply[1], login::OK);
        let first = &server.worlds.all()[0];
        assert_eq!((first.world.seats().online(), second.world.seats().online()), (0, 1));

        assert_eq!(server.worlds.save(), Ok(()));
        assert_eq!(server.worlds.saved(), vec![options]);
    }
}
