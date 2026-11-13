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
    // The self-test's own player account mustn't land in the real accounts file.
    settings_used.accounts_file = None;
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

    // The login window from the cache, fetched over JS5 and drawn by its own scripts.
    let window = app.window.as_mut().map(|w| {
        w.render();
        let lit = w.raster.pixels.iter().filter(|&&p| p != 0).count();
        let picture = Path::new(out).with_file_name("login-window.png");
        if let Err(e) = w.raster.save_png(&picture) {
            eprintln!("couldn't save {}: {e}", picture.display());
        }
        lit
    });
    steps.push(Step {
        name: "login-window",
        passed: window.is_some_and(|lit| lit > 100_000),
        detail: match window {
            Some(lit) => format!("login window opened from the cache over JS5, its on-load scripts run; {lit} lit pixels"),
            None => "the login window didn't open".into(),
        },
    });

    // The login music (`GameState.setGameState` → `Class339.method6047`): it fades in from
    // nothing, so the player loads it, renders it in the background and plays it. Waits for it
    // (up to 60 s: rendering the whole song takes a while in a debug build).
    let music = app.window.as_mut().map(|w| {
        let started = Instant::now();
        while w.ui.jukebox.status() != "playing" && started.elapsed().as_secs() < 60 {
            w.ui.tick_music();
            std::thread::sleep(Duration::from_millis(20));
        }
        (w.ui.jukebox.status(), started.elapsed().as_secs_f32(), w.ui.jukebox.song)
    });
    steps.push(Step {
        name: "login-music",
        passed: music.is_some_and(|(status, ..)| status == "playing"),
        detail: match music {
            Some((status, secs, song)) => format!("login music (track {song}) {status} after {secs:.1} s"),
            None => "no login window".into(),
        },
    });

    // Logging in as a player would: on the login window from the cache, answer the graphics
    // dialog with "Auto Setup" (it times the graphics card), click each box and type, click "Log In"; the window's own
    // scripts read the boxes and ask to log in. Without that window, the plain form.
    // On the way, open the options (the wrench), drag the music slider's knob to the far
    // left (the window's drag-release script sets the music volume) and close them again.
    let mut music_volume = None;
    let mut setup = None;
    match app.window.as_mut() {
        Some(w) => {
            if let Err(e) = w.ui.play("click 380 272") {
                eprintln!("auto setup input: {e}");
            }
            w.render();
            let picture = Path::new(out).with_file_name("auto-setup.png");
            if let Err(e) = w.raster.save_png(&picture) {
                eprintln!("couldn't save {}: {e}", picture.display());
            }
            // A new toolkit asks "Your display has been changed... keep this setting?": Yes.
            if w.ui.prefs.current_toolkit != 0 {
                if let Err(e) = w.ui.play("click 332 290") {
                    eprintln!("display check input: {e}");
                }
            }
            setup = Some((w.ui.prefs.graphics, w.ui.prefs.toolkit, w.ui.prefs.current_toolkit));
            let options = ["click 721 16", "drag 312 345 262 345"];
            let login = ["click 619 63", "click 380 186", "type selftest", "click 380 233", "type password1", "click 381 274"];
            if let Some(e) = options.iter().find_map(|s| w.ui.play(s).err()) {
                eprintln!("options panel input: {e}");
            }
            music_volume = Some(w.ui.prefs.music_volume);
            if let Some(e) = login.iter().find_map(|s| w.ui.play(s).err()) {
                eprintln!("login window input: {e}");
            }
        }
        None => {
            app.type_text("selftest");
            app.key(Key::Tab);
            app.type_text("password1");
            app.key(Key::Enter);
        }
    }
    steps.push(Step {
        name: "auto-setup",
        passed: setup.is_some_and(|(level, ..)| (1..=4).contains(&level)),
        detail: match setup {
            Some((level, toolkit, current)) => format!(
                "clicked Auto Setup on the login window: detail level {level} (1 MIN .. 4 HIGH), saved toolkit {toolkit}, toolkit in use {current}"
            ),
            None => "no login window".into(),
        },
    });
    steps.push(Step {
        name: "volume-slider",
        passed: music_volume == Some(0),
        detail: match music_volume {
            Some(v) => format!("dragged the music slider in the login options to the left: music volume {v} (was 127)"),
            None => "no login window to drag in".into(),
        },
    });
    let logged_in = frames_until(&mut app, |a| matches!(a.stage, Stage::World { .. } | Stage::Lobby(..))).await;
    let logins = server.stats.lock().unwrap().logins;
    // In the lobby: wait for the server's "lobby ready" variable (2567), then let the window's
    // scripts build it and fade it in (script 5954, about 150 ticks) before the picture.
    if matches!(app.stage, Stage::Lobby(..)) {
        let ready = frames_until(&mut app, |a| a.window.as_ref().is_some_and(|w| w.ui.varp(2567) != 0)).await;
        let ready_at = app.window.as_ref().map_or(0, |w| w.ui.clock());
        frames_until(&mut app, |a| a.window.as_ref().is_some_and(|w| w.ui.clock() > ready_at + 200)).await;
        steps.push(Step {
            name: "lobby-packets",
            passed: ready,
            detail: if ready {
                "the lobby connection stayed open and the server's player variable 2567 arrived".into()
            } else {
                "no lobby packets arrived".into()
            },
        });
    }
    if let (Stage::Lobby(..), Some(w)) = (&app.stage, app.window.as_mut()) {
        w.render();
        let picture = Path::new(out).with_file_name("lobby.png");
        if let Err(e) = w.raster.save_png(&picture) {
            eprintln!("couldn't save {}: {e}", picture.display());
        }
    }
    steps.push(Step {
        name: "act",
        passed: logged_in && logins == 1,
        detail: match &app.stage {
            Stage::Lobby(details, _) => format!(
                "typed name and password into the login window and clicked Log In: lobby login as '{}' (member: {}), the lobby window opened; server counted {logins} login",
                details.name, details.member
            ),
            Stage::World { name, player_index, arrival } => format!(
                "typed name and password into the login window and clicked Log In: logged in as '{name}', player {player_index}, first game packet REGION put the player at {:?} with keys for {} map regions; server counted {logins} login",
                arrival.at, arrival.regions
            ),
            Stage::Login(screen) => format!("login screen says: {}", screen.message.0),
            Stage::Loading(_) => "still loading".into(),
        },
    });

    // Log out: the lobby's close button runs `instr6623`; the client closes the lobby and opens
    // the login window again. Then log in again the same way, back to the lobby.
    if let (Stage::Lobby(..), Some(w)) = (&app.stage, app.window.as_mut()) {
        if let Err(e) = w.ui.play("click 689 33") {
            eprintln!("lobby input: {e}");
        }
        let left = frames_until(&mut app, |a| matches!(a.stage, Stage::Login(_))).await;
        let login_window = app.window.as_mut().and_then(|w| w.ui.assets.login_window().ok());
        let root_is_login = app.window.as_ref().zip(login_window).is_some_and(|(w, id)| w.ui.root_id() == Some(id));
        if let Some(w) = app.window.as_mut() {
            w.render();
            let picture = Path::new(out).with_file_name("logged-out.png");
            if let Err(e) = w.raster.save_png(&picture) {
                eprintln!("couldn't save {}: {e}", picture.display());
            }
            for s in ["click 380 186", "type selftest", "click 380 233", "type password1", "click 381 274"] {
                if let Err(e) = w.ui.play(s) {
                    eprintln!("login window input: {e}");
                }
            }
        }
        let back = frames_until(&mut app, |a| matches!(a.stage, Stage::Lobby(..))).await;
        frames_until(&mut app, |a| a.window.as_ref().is_some_and(|w| w.ui.varp(2567) != 0)).await;
        let ready_at = app.window.as_ref().map_or(0, |w| w.ui.clock());
        frames_until(&mut app, |a| a.window.as_ref().is_some_and(|w| w.ui.clock() > ready_at + 200)).await;
        steps.push(Step {
            name: "logout",
            passed: left && root_is_login && back,
            detail: format!(
                "clicked the lobby's close button: back on the login screen: {left}, login window open: {root_is_login}; logged in again to the lobby: {back}"
            ),
        });
    }

    // Play: the lobby's PLAY button runs `instr6641`, and the client logs in to the world with
    // the same name and password (game state 7 → `worldLogin`), then reads `REGION`.
    if let (Stage::Lobby(..), Some(w)) = (&app.stage, app.window.as_mut()) {
        if let Err(e) = w.ui.play("click 381 461") {
            eprintln!("lobby input: {e}");
        }
        let in_world = frames_until(&mut app, |a| matches!(a.stage, Stage::World { .. })).await;
        let logins = server.stats.lock().unwrap().logins;
        // A picture of the world as drawn (the ground around the player).
        if in_world {
            for _ in 0..3 {
                app.update();
                app.draw();
                next_frame().await;
            }
            app.draw();
            let picture = Path::new(out).with_file_name("world.png");
            get_screen_data().export_png(&picture.to_string_lossy());
        }
        steps.push(Step {
            name: "play",
            passed: in_world && logins == 3,
            detail: match &app.stage {
                Stage::World { name, player_index, arrival } => format!(
                    "clicked PLAY in the lobby: world login as '{name}', player {player_index}, REGION put the player at {:?} with keys for {} map regions; server counted {logins} logins",
                    arrival.at, arrival.regions
                ),
                _ => format!("still in the lobby after PLAY (login reply {})", app.window.as_ref().map_or(0, |w| w.ui.login_reply)),
            },
        });
    }

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

    // Logged in means the lobby (with `lobby = true`) or the world.
    let d = app.drawn;
    let logged_in = d.lobby || d.world;
    steps.push(Step {
        name: "hud",
        passed: d.loading && d.login && logged_in,
        detail: format!(
            "screens drawn - loading: {}, login: {}, logged in: {logged_in} (lobby: {}, world: {})",
            d.loading, d.login, d.lobby, d.world
        ),
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
