//! `--self-test <file.json>`: starts a server (real cache from server.cfg, or a
//! sample), then drives the real client screens with scripted input — loading,
//! typing a name and password, logging in — and records whether each step worked.

mod sample;

use crate::client::login_screen::Key;
use crate::client::{App, Stage};
use crate::config::Settings;
use crate::server_app;
use macroquad::prelude::*;
use std::path::Path;
use std::time::{Duration, Instant};

const STEP_TIMEOUT: Duration = Duration::from_secs(120);

struct Step {
    name: &'static str,
    passed: bool,
    detail: String,
}

/// Runs frames until `done` holds or the timeout passes. Returns whether it held.
async fn frames_until(app: &mut App, mut done: impl FnMut(&App) -> bool) -> bool {
    let started = Instant::now();
    while started.elapsed() < STEP_TIMEOUT {
        app.update();
        app.draw();
        next_frame().await;
        if done(app) {
            return true;
        }
    }
    false
}

pub async fn run(settings: &Settings, out: &str) {
    let mut steps = Vec::new();
    let mut settings_used = settings.clone();
    let real = settings.cache_dir.join(crate::cache::store::DATA_FILE).is_file();
    if !real {
        match sample::write(&std::env::temp_dir().join("cache-server-selftest")) {
            Ok((cache, keys)) => (settings_used.cache_dir, settings_used.keys_file) = (cache, keys),
            Err(e) => {
                steps.push(Step { name: "open", passed: false, detail: format!("could not write the sample cache: {e}") });
                return save(out, &steps);
            }
        }
    }
    let which = if real { "real cache" } else { "sample cache" };

    // The server, on a free local port.
    let (server, address, loader) = match server_app::start_now(&settings_used, "127.0.0.1:0") {
        Ok(s) => s,
        Err(e) => {
            steps.push(Step { name: "open", passed: false, detail: format!("server didn't start: {e}") });
            return save(out, &steps);
        }
    };
    let tables_ok = !loader.indexes.is_empty() && loader.indexes.iter().all(|i| i.result.is_ok());
    steps.push(Step { name: "open", passed: tables_ok, detail: format!("server on {address} with the {which}: {}", loader.status()) });
    let r = &loader.key_report;
    steps.push(Step {
        name: "keys",
        passed: r.error.is_none() && r.ok > 0 && r.wrong == 0,
        detail: format!(
            "{} map keys, {} decrypt their archive, {} wrong, {} not in cache {}",
            r.total,
            r.ok,
            r.wrong,
            r.missing,
            r.error.clone().unwrap_or_default()
        ),
    });

    // The client: loading screen until the cache is fetched and checked.
    let mut app = App::new(&settings_used, address.clone());
    let loaded = frames_until(&mut app, |a| !matches!(a.stage, Stage::Loading(_))).await;
    let summary = app.summary.clone();
    steps.push(Step {
        name: "move",
        passed: loaded && summary.is_some(),
        detail: match &summary {
            Some(s) => format!("loading screen reached 100%: {} reference tables and {} config archives ({} files) downloaded and CRC-checked ({} bytes), then the login screen opened", s.indexes, s.archives, s.files, s.bytes),
            None => format!("loading didn't finish: {}", loading_text(&app)),
        },
    });

    // Typing on the login screen exactly as a player would.
    app.type_text("selftest");
    app.key(Key::Tab);
    app.type_text("password1");
    app.key(Key::Enter);
    let logged_in = frames_until(&mut app, |a| matches!(a.stage, Stage::World { .. })).await;
    let logins = server.stats.lock().unwrap().logins;
    steps.push(Step {
        name: "act",
        passed: logged_in && logins == 1,
        detail: match &app.stage {
            Stage::World { name, player_index } => format!(
                "typed name and password, pressed Enter: logged in as '{name}', player {player_index}; server counted {logins} login"
            ),
            Stage::Login(screen) => format!("login screen says: {}", screen.message.0),
            Stage::Loading(_) => "still loading".into(),
        },
    });

    // The cache's loading screens: every screen in the running order, drawn offscreen.
    let screen_count = app.loading_screens.screen_count();
    let status = crate::client::loading_screen::sample_status();
    let lit: Vec<usize> = (0..screen_count).filter_map(|n| app.loading_screens.lit_pixels_of_screen(n, &status)).collect();
    let screens_ok = screen_count > 0 && lit.len() == screen_count && lit.iter().all(|&n| n > 1000);
    // Keep a picture of the busiest screen next to the results.
    if let Some(busiest) = (0..lit.len()).max_by_key(|&n| lit[n]) {
        app.loading_screens.lit_pixels_of_screen(busiest, &status);
        let picture = Path::new(out).with_file_name("loading-screen.png");
        if let Err(e) = app.loading_screens.save_raster_png(&picture) {
            eprintln!("couldn't save {}: {e}", picture.display());
        }
    }
    steps.push(Step {
        name: "loading-screens",
        passed: screens_ok,
        detail: format!("{screen_count} loading screens from index 33 decoded with their images and fonts; lit pixels per screen: {lit:?}"),
    });

    let d = app.drawn;
    steps.push(Step {
        name: "hud",
        passed: d.loading && d.login && d.world,
        detail: format!("screens drawn - loading: {}, login: {}, logged in: {}", d.loading, d.login, d.world),
    });
    save(out, &steps);
}

fn loading_text(app: &App) -> String {
    match &app.stage {
        Stage::Loading(p) => {
            let p = p.lock().unwrap();
            match &p.result {
                Some(Err(e)) => e.clone(),
                _ => p.status.text.clone(),
            }
        }
        _ => String::new(),
    }
}

fn save(out: &str, steps: &[Step]) {
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    let rows: Vec<String> = steps
        .iter()
        .map(|s| format!("    {{\"step\": \"{}\", \"passed\": {}, \"detail\": \"{}\"}}", s.name, s.passed, esc(&s.detail)))
        .collect();
    let json = format!("{{\"version\": 1, \"steps\": [\n{}\n]}}\n", rows.join(",\n"));
    print!("{json}");
    if let Some(dir) = Path::new(out).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(out, json) {
        eprintln!("could not write {out}: {e}");
    }
}
