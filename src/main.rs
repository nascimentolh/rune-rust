//! Game client and game server in one program, always run as two separate processes.
//!
//! - Opening the program starts the CLIENT: loading screen (fetches the cache from
//!   the server), then the login screen. If no server is running it starts one.
//! - `--server` runs the SERVER: loads the cache and map keys, serves the cache
//!   (JS5) and logins on one port, with a status window (`--console` for a terminal).
//!
//! Modules:
//! - `client`     — the client (updater, login, screens, server launcher)
//! - `server_app` — running the server process
//! - `config`     — server.cfg settings, shared by both
//! - `cache`      — the cache format (files, containers, reference tables, archives, XTEA, keys)
//! - `loader`     — server start-up sequence
//! - `net`        — the server port: one thread per client, picks the service
//! - `js5`        — the cache service (protocol, checksum table)
//! - `login`      — the login service (login packet, RSA, ISAAC)
//! - `trace`      — byte-level log of every server connection
//! - `screen`     — the server's status window
//! - `selftest`   — scripted run of client and server for automatic checks
//!
//! Usage: app [--config server.cfg] [--server [--console]] [--gen-rsa] [--screenshot <png>] [--self-test <json>]

mod cache;
mod client;
mod config;
mod js5;
mod loader;
mod login;
mod net;
mod screen;
mod selftest;
mod server_app;
mod tools;
mod trace;

use config::Settings;
use macroquad::prelude::*;

const SCREENSHOT_FRAMES: u32 = 120;

/// A developer tool run from the command line with one numeric id.
type Tool = fn(&Settings, u32) -> Result<(), String>;

fn window(title: &str, w: f32, h: f32) -> Conf {
    Conf { window_title: title.into(), window_width: w as i32, window_height: h as i32, window_resizable: false, ..Default::default() }
}

fn arg_after(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
}

fn has_flag(flag: &str) -> bool {
    std::env::args().any(|a| a == flag)
}

fn load_settings() -> Settings {
    let path = arg_after("--config").map(Into::into).or_else(config::find);
    match path {
        Some(p) => Settings::load(&p).unwrap_or_else(|e| {
            eprintln!("Bad settings, using defaults: {e}");
            Settings::default()
        }),
        None => {
            eprintln!("No {} found, using defaults", config::FILE_NAME);
            Settings::default()
        }
    }
}

async fn run_client(settings: Settings) {
    if let Some(out) = arg_after("--self-test") {
        return selftest::run(&settings, &out).await;
    }
    let screenshot = arg_after("--screenshot");
    // A screenshot run gets its own server on a free port, inside this process,
    // so it never depends on (or clashes with) a server that's already running.
    let address = match &screenshot {
        Some(_) => match server_app::start_now(&settings, "127.0.0.1:0") {
            Ok((server, address, _)) => {
                std::mem::forget(server); // keep it serving until this run exits
                address
            }
            Err(e) => {
                eprintln!("{e}");
                settings.server_address()
            }
        },
        None => {
            client::launch::ensure_server(&settings.server_address());
            settings.server_address()
        }
    };

    let mut app = client::App::new(&settings, address);
    let mut frame = 0;
    loop {
        app.handle_input();
        app.update();
        app.draw();
        frame += 1;
        if let Some(path) = &screenshot {
            if frame >= SCREENSHOT_FRAMES {
                if let Some(dir) = std::path::Path::new(path).parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                get_screen_data().export_png(path);
                return;
            }
        }
        next_frame().await;
    }
}

fn main() {
    if has_flag("--gen-rsa") {
        return server_app::print_new_rsa_keys();
    }
    let settings = load_settings();
    let tools: [(&str, Tool); 5] = [
        ("--export-music", tools::export_music),
        ("--dump-interface", tools::dump_interface),
        ("--dump-script", tools::dump_script),
        ("--script-scope", tools::script_scope),
        ("--render-interface", tools::render_interface),
    ];
    for (flag, tool) in tools {
        if let Some(id) = arg_after(flag) {
            if let Err(e) = id.parse().map_err(|_| format!("{flag} needs a number")).and_then(|id| tool(&settings, id)) {
                eprintln!("{e}");
            }
            return;
        }
    }
    if has_flag("--server") {
        if has_flag("--console") {
            server_app::run_console(settings);
        } else {
            macroquad::Window::from_config(window("Game server", screen::SCREEN_W, screen::SCREEN_H), server_app::run_window(settings));
        }
    } else {
        macroquad::Window::from_config(window("Game client", client::WIDTH, client::HEIGHT), run_client(settings));
    }
}
