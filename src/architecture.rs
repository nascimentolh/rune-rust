//! The rules of `docs/SERVER.md` that can be checked by reading the source, checked. When one
//! of these fails, the fix is to move the code where it belongs, not to change the test.

use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every line of code (not comment) under `src/<folder>` holding one of `words`, as
/// "file:line: text", leaving out the files named in `except`.
fn uses(folder: &str, words: &[&str], except: &[&str]) -> Vec<String> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src.join(folder), &mut files);
    let mut found = Vec::new();
    for file in files {
        let name = file.strip_prefix(&src).unwrap().to_string_lossy().replace('\\', "/");
        if except.contains(&name.as_str()) {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if !code.starts_with("//") && words.iter().any(|w| code.contains(w)) {
                found.push(format!("src/{name}:{}: {}", n + 1, code));
            }
        }
    }
    found
}

fn none(found: Vec<String>, rule: &str) {
    assert!(found.is_empty(), "{rule} (docs/SERVER.md):\n{}", found.join("\n"));
}

#[test]
fn the_client_and_the_server_meet_only_in_the_protocol() {
    none(uses("server", &["crate::client"], &[]), "the server never uses the client");
    none(uses("client", &["crate::server"], &[]), "the client never uses the server");
    none(uses("protocol", &["crate::client", "crate::server"], &[]), "the protocol belongs to neither side");
    none(uses("cache", &["crate::client", "crate::server"], &[]), "the cache formats belong to neither side");
}

#[test]
fn only_the_worlds_thread_touches_game_state() {
    none(
        uses("server/world", &["Mutex", "RwLock", "Atomic"], &["server/world/seats.rs"]),
        "game state is owned by the world's thread, never shared behind a lock",
    );
    none(
        uses("server/world", &["TcpStream", "write_all", "read_exact", "thread::sleep"], &["server/world/connection.rs"]),
        "only world/connection.rs touches a socket, and nothing in the world waits",
    );
}

#[test]
fn payloads_are_built_and_read_only_in_the_protocol() {
    // A payload is bytes put together by hand; outside `protocol` they come from an encoder.
    none(
        uses("server/world", &["to_be_bytes", "to_le_bytes", "from_be_bytes", "from_le_bytes"], &[]),
        "the world builds and reads payloads with crate::protocol's encoders and decoders",
    );
    none(
        uses("server/lobby", &["to_be_bytes", "to_le_bytes", "from_be_bytes", "from_le_bytes"], &[]),
        "the lobby builds and reads payloads with crate::protocol's encoders and decoders",
    );
}
