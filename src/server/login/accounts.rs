//! Player accounts, server side. There is no reference server, so the rules are this
//! server's own: the first login with a name creates the account with that password; later
//! logins need the same password (else reply 3, "Invalid login or password"); a name already
//! in the same place (the lobby, or the world) is refused with reply 5. Lobby and world are
//! separate places, as the client logs in to the world from the lobby (`Class78.method1384`)
//! while its lobby connection is still open. Names are matched without case and surrounding
//! spaces.
//!
//! `log_in` gives back an `Online`: the name stays in its place for as long as that is held,
//! and leaves it when it is dropped, however the session ends.
//!
//! Saved one per line as `name salt hash` (hex) in the accounts file; the password itself is
//! never stored, only Whirlpool(salt ‖ password). `None` keeps the accounts in memory only
//! (the self-test).

use crate::protocol::login::{ALREADY_ONLINE, INVALID_LOGIN};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use whirlpool::{Digest, Whirlpool};

pub struct Accounts {
    file: Option<PathBuf>,
    /// Name → (salt, hash).
    saved: Mutex<HashMap<String, (String, String)>>,
    /// (name, in the world): who is in the lobby or the world now.
    online: Mutex<HashSet<(String, bool)>>,
}

/// A name's presence in the lobby or the world; dropping it is the player leaving.
pub struct Online {
    accounts: Arc<Accounts>,
    name: String,
    world: bool,
}

impl Drop for Online {
    fn drop(&mut self) {
        self.accounts.online.lock().unwrap().remove(&(std::mem::take(&mut self.name), self.world));
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hash(salt: &str, password: &str) -> String {
    hex(&Whirlpool::digest(format!("{salt}{password}").as_bytes()))
}

/// The account a login name refers to.
fn key(name: &str) -> String {
    name.trim().to_lowercase()
}

impl Accounts {
    pub fn new(file: Option<PathBuf>) -> Arc<Self> {
        let saved = file
            .as_ref()
            .and_then(|f| std::fs::read_to_string(f).ok())
            .map(|text| {
                text.lines()
                    .filter_map(|l| {
                        let mut p = l.split_whitespace();
                        Some((p.next()?.to_string(), (p.next()?.to_string(), p.next()?.to_string())))
                    })
                    .collect()
            })
            .unwrap_or_default();
        Arc::new(Accounts { file, saved: Mutex::new(saved), online: Mutex::new(HashSet::new()) })
    }

    /// Checks a login and marks the name as in the lobby or (`world`) the world until the
    /// `Online` it returns is dropped; `Err(code)` refuses it.
    pub fn log_in(self: &Arc<Self>, name: &str, password: &str, world: bool) -> Result<Online, u8> {
        let name = key(name);
        let mut saved = self.saved.lock().unwrap();
        match saved.get(&name) {
            Some((salt, h)) if *h != hash(salt, password) => return Err(INVALID_LOGIN),
            Some(_) => {}
            None => {
                let salt = hex(&rand::random::<[u8; 16]>());
                let h = hash(&salt, password);
                saved.insert(name.clone(), (salt, h));
                self.save(&saved);
            }
        }
        drop(saved);
        if !self.online.lock().unwrap().insert((name.clone(), world)) {
            return Err(ALREADY_ONLINE);
        }
        Ok(Online { accounts: Arc::clone(self), name, world })
    }

    fn save(&self, saved: &HashMap<String, (String, String)>) {
        let Some(file) = &self.file else { return };
        let mut lines: Vec<String> = saved.iter().map(|(n, (s, h))| format!("{n} {s} {h}")).collect();
        lines.sort();
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(file, lines.join("\n") + "\n") {
            eprintln!("couldn't save accounts to {}: {e}", file.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_login_creates_then_the_password_must_match() {
        let a = Accounts::new(None);
        let bob = a.log_in("Bob", "secret", true);
        assert!(bob.is_ok());
        assert_eq!(a.log_in(" bob ", "secret", true).err(), Some(ALREADY_ONLINE));
        drop(bob);
        assert_eq!(a.log_in("bob", "wrong", true).err(), Some(INVALID_LOGIN));
        assert!(a.log_in("bob", "secret", true).is_ok());
    }

    #[test]
    fn the_lobby_and_the_world_are_separate_places() {
        let a = Accounts::new(None);
        let lobby = a.log_in("ann", "pw", false);
        let world = a.log_in("ann", "pw", true);
        assert!(lobby.is_ok() && world.is_ok());
        assert_eq!(a.log_in("ann", "pw", false).err(), Some(ALREADY_ONLINE));
    }
}
