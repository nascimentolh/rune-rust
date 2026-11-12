//! Player accounts, server side. There is no reference server, so the rules are this
//! server's own: the first login with a name creates the account with that password; later
//! logins need the same password (else reply 3, "Invalid login or password"); a name already
//! in the game is refused with reply 5. Names are matched without case and surrounding spaces.
//!
//! Saved one per line as `name salt hash` (hex) in the accounts file; the password itself is
//! never stored, only Whirlpool(salt ‖ password). `None` keeps the accounts in memory only
//! (the self-test).

use super::protocol::{ALREADY_ONLINE, INVALID_LOGIN};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Mutex;
use whirlpool::{Digest, Whirlpool};

pub struct Accounts {
    file: Option<PathBuf>,
    /// Name → (salt, hash).
    saved: Mutex<HashMap<String, (String, String)>>,
    online: Mutex<HashSet<String>>,
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
    pub fn new(file: Option<PathBuf>) -> Self {
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
        Accounts { file, saved: Mutex::new(saved), online: Mutex::new(HashSet::new()) }
    }

    /// Checks a login and marks the name as in the game; `Err(code)` refuses it.
    pub fn log_in(&self, name: &str, password: &str) -> Result<(), u8> {
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
        if !self.online.lock().unwrap().insert(name) {
            return Err(ALREADY_ONLINE);
        }
        Ok(())
    }

    /// The player left the game.
    pub fn log_out(&self, name: &str) {
        self.online.lock().unwrap().remove(&key(name));
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
        assert_eq!(a.log_in("Bob", "secret"), Ok(()));
        assert_eq!(a.log_in(" bob ", "secret"), Err(ALREADY_ONLINE));
        a.log_out("BOB");
        assert_eq!(a.log_in("bob", "wrong"), Err(INVALID_LOGIN));
        assert_eq!(a.log_in("bob", "secret"), Ok(()));
    }
}
