//! Running the server process: load the cache, start both services, and show
//! progress either in its status window or in a terminal (`--console`).

use crate::config::Settings;
use crate::loader::{self, Loader};
use crate::net::Server;
use crate::{js5, login, screen, trace};
use macroquad::prelude::*;
use std::net::TcpListener;
use std::sync::Arc;

/// After loading: build the checksum table, set up both services and start listening.
pub fn start_server(loader: &mut Loader, settings: &Settings, bind: &str) -> Result<(Arc<Server>, String), String> {
    let revisions = loader.revisions();
    let mut store = loader.cache.take().ok_or("the cache didn't load")?.store;
    let checksums = js5::index_checksums(&mut store, &revisions);
    let table = crate::protocol::checksum_table::build(&checksums, settings.js5_rsa.as_ref());
    let trace = match &settings.trace_file {
        Some(path) => Some(trace::TraceFile::open(path).map_err(|e| format!("can't open trace {}: {e}", path.display()))?),
        None => None,
    };
    let listener = TcpListener::bind(bind).map_err(|e| format!("can't listen on {bind}: {e}"))?;
    let address = listener.local_addr().map_err(|e| e.to_string())?.to_string();
    let server =
        Arc::new(Server::new(js5::Js5::new(store, table, settings), login::LoginService::new(settings, loader.keys.by_region()), trace));
    server.start(listener);
    Ok((server, address))
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

/// The server's status window: loading progress, then connections and log.
pub async fn run_window(settings: Settings) {
    let bind = format!("0.0.0.0:{}", settings.port);
    let mut loader = Loader::new(settings.cache_dir.clone(), settings.keys_file.clone());
    let mut server: Option<(Arc<Server>, String)> = None;
    loop {
        if !loader.finished() {
            loader.step();
            if loader.finished() && loader.cache.is_some() {
                match start_server(&mut loader, &settings, &bind) {
                    Ok(s) => server = Some(s),
                    Err(e) => loader.stage = loader::Stage::Failed(e),
                }
            }
        }
        screen::draw(&loader, server.as_ref().map(|(s, a)| (s.as_ref(), a.as_str())));
        next_frame().await;
    }
}
