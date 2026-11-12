//! Server settings, read from `server.cfg` (`key = value` lines, `#` starts a comment line).
//! Relative paths are relative to the settings file's folder.

use num_bigint::BigUint;
use std::path::{Path, PathBuf};

pub const FILE_NAME: &str = "server.cfg";

#[derive(Clone)]
pub struct Settings {
    pub cache_dir: PathBuf,
    pub keys_file: PathBuf,
    /// Where the client finds the server.
    pub host: String,
    pub port: u16,
    /// Build the client must send (when `check_build` is on).
    pub build: i32,
    pub sub_build: i32,
    pub check_build: bool,
    /// Key the client sends in the cache handshake (client parameter 17).
    pub js5_key: String,
    /// Sent to the client right after a good cache handshake.
    pub grab_keys: Vec<i32>,
    /// Private key for signing the checksum table; `None` = send it unsigned.
    pub js5_rsa: Option<Rsa>,
    /// Private key for the secret part of the login; `None` = the client sends it unencrypted.
    pub login_rsa: Option<Rsa>,
    /// Where to write the byte-level trace; `None` = no trace.
    pub trace_file: Option<PathBuf>,
    /// Where player accounts are saved; `None` = in memory only.
    pub accounts_file: Option<PathBuf>,
    /// The client logs in to the lobby first (`Loader.LOBBY_ENABLED`). The reference has it
    /// off; on here because the lobby is wanted.
    pub lobby: bool,
    /// Show the lobby's top bar (Home / Support / Forums / Clans, component 906:1 of the cache).
    /// The reference shows it; off here because this cache's bar is a custom one.
    pub lobby_top_bar: bool,
    /// Most players in the world at once (at most 2047, the client's player table).
    pub world_capacity: u16,
}

#[derive(Clone)]
pub struct Rsa {
    pub exponent: BigUint,
    pub modulus: BigUint,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            cache_dir: "cache".into(),
            keys_file: "data/keys.json".into(),
            host: "127.0.0.1".into(),
            // Loader.java: WORLD_PORT = LOBBY_PORT = 5555, MAJOR_BUILD = 727, MINOR_BUILD = 1.
            port: 5555,
            build: 727,
            sub_build: 1,
            check_build: false,
            // Loader.java: clientParams "17".
            js5_key: "ev9+VAp5/tMKeNR/7MOuH6lKWS+rGkHK".into(),
            grab_keys: Vec::new(),
            js5_rsa: None,
            login_rsa: None,
            trace_file: None,
            accounts_file: Some("data/accounts.txt".into()),
            lobby: true,
            lobby_top_bar: false,
            world_capacity: 2000,
        }
    }
}

/// Finds `server.cfg`: current folder first, then next to the program, then two folders up (target/debug → project).
pub fn find() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    [PathBuf::from(FILE_NAME), exe_dir.join(FILE_NAME), exe_dir.join("../..").join(FILE_NAME)].into_iter().find(|p| p.is_file())
}

/// An RSA key given as two settings; both or neither must be set.
#[derive(Default)]
struct KeyParts {
    exponent: Option<BigUint>,
    modulus: Option<BigUint>,
}

impl KeyParts {
    fn finish(self, name: &str) -> Result<Option<Rsa>, String> {
        match (self.exponent, self.modulus) {
            (Some(exponent), Some(modulus)) => Ok(Some(Rsa { exponent, modulus })),
            (None, None) => Ok(None),
            _ => Err(format!("{name}_exponent and {name}_modulus must both be set, or both left empty")),
        }
    }
}

impl Settings {
    /// host:port the client connects to.
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let base = path.parent().unwrap_or(Path::new("."));
        Self::parse(&text, base)
    }

    pub fn parse(text: &str, base: &Path) -> Result<Self, String> {
        let mut s = Settings::default();
        let (mut js5_key, mut login_key) = (KeyParts::default(), KeyParts::default());
        for (n, line) in text.lines().enumerate() {
            // Only whole-line comments: paths may contain '#'.
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line.split_once('=').ok_or(format!("line {}: expected key = value", n + 1))?;
            let (key, value) = (key.trim(), value.trim());
            let bad = |what: &str| format!("line {}: {key} must be {what}", n + 1);
            let big = || parse_big(value).ok_or(bad("a decimal or 0x-hex number"));
            // A setting left empty keeps its default.
            if value.is_empty() {
                continue;
            }
            match key {
                "cache" => s.cache_dir = base.join(value),
                "keys" => s.keys_file = base.join(value),
                "trace" => s.trace_file = Some(base.join(value)),
                "accounts" => s.accounts_file = Some(base.join(value)),
                "world_capacity" => s.world_capacity = value.parse().map_err(|_| bad("a number of players"))?,
                "lobby" => s.lobby = value.parse().map_err(|_| bad("true or false"))?,
                "lobby_top_bar" => s.lobby_top_bar = value.parse().map_err(|_| bad("true or false"))?,
                "host" => s.host = value.to_string(),
                "port" => s.port = value.parse().map_err(|_| bad("a port number"))?,
                "build" => s.build = value.parse().map_err(|_| bad("a number"))?,
                "sub_build" => s.sub_build = value.parse().map_err(|_| bad("a number"))?,
                "js5_key" => s.js5_key = value.to_string(),
                "check_build" => s.check_build = value.parse().map_err(|_| bad("true or false"))?,
                "grab_keys" => {
                    s.grab_keys = value
                        .split(',')
                        .filter(|v| !v.trim().is_empty())
                        .map(|v| v.trim().parse())
                        .collect::<Result<_, _>>()
                        .map_err(|_| bad("numbers separated by commas"))?
                }
                "rsa_exponent" => js5_key.exponent = Some(big()?),
                "rsa_modulus" => js5_key.modulus = Some(big()?),
                "login_rsa_exponent" => login_key.exponent = Some(big()?),
                "login_rsa_modulus" => login_key.modulus = Some(big()?),
                other => return Err(format!("line {}: unknown setting '{other}'", n + 1)),
            }
        }
        s.js5_rsa = js5_key.finish("rsa")?;
        s.login_rsa = login_key.finish("login_rsa")?;
        Ok(s)
    }
}

fn parse_big(v: &str) -> Option<BigUint> {
    match v.strip_prefix("0x") {
        Some(hex) => BigUint::parse_bytes(hex.as_bytes(), 16),
        None => BigUint::parse_bytes(v.as_bytes(), 10),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_setting() {
        let s = Settings::parse(
            "# hi\ncache = c#302\nport = 1\nbuild = 727\ncheck_build = true\ngrab_keys = 1, 2,3\nrsa_exponent = 0x11\nrsa_modulus = 99\nlogin_rsa_exponent =\nlogin_rsa_modulus =\ntrace = logs/t.log\n",
            Path::new("base"),
        )
        .unwrap();
        assert_eq!(s.cache_dir, Path::new("base").join("c#302"));
        assert_eq!((s.port, s.build, s.check_build), (1, 727, true));
        assert_eq!(s.grab_keys, vec![1, 2, 3]);
        assert_eq!(s.js5_rsa.unwrap().exponent, BigUint::from(17u32));
        assert!(s.login_rsa.is_none());
        assert_eq!(s.trace_file, Some(Path::new("base").join("logs/t.log")));
        assert!(Settings::parse("nope = 1", Path::new(".")).is_err());
        assert!(Settings::parse("rsa_modulus = 5", Path::new(".")).is_err());
    }
}
