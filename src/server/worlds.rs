//! The game worlds this server runs: the first one, made from `server.cfg`, and any the
//! administrator opens from the status window. Each has its own port, its own thread and its
//! own players (`super::world`); nothing is shared between two worlds.
//!
//! There is no reference server, so what a world can be given is this server's choice, kept to
//! what the client can be told about a world: the fields and flag bits of the world list
//! (`crate::protocol::world_list`) and the two the world-login reply carries
//! (`crate::protocol::login::WorldInfo`).
//!
//! Worlds opened by the administrator are saved as JSON (`worlds` in `server.cfg`) and opened
//! again when the server starts. To remove one, delete it from that file while the server is
//! stopped; a world in it that can't be used (a port already taken, say) is left out with a
//! line in the log.

use super::net::SharedStats;
use super::world::{World, MAX_PLAYERS};
use crate::cache::xtea::Key;
use crate::protocol::login::WorldInfo;
use crate::protocol::world_list::{BOTS_ALLOWED, HIDDEN, HIGHLIGHTED, LOOTSHARE, MEMBERS, QUICK_CHAT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// The longest a world's name and its location's name may be. This server's choice: what fits
/// the World Select column they share.
pub const NAME_LEN: usize = 24;

/// What the administrator chooses for a world. Everything here either goes in the world list
/// or in the login reply; nothing is kept that the client is never told.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Options {
    /// The world's activity text. The World Select shows it in the Location/Activity column
    /// *instead of* the flag and the location's name (script 3117), so a world with a name
    /// never shows its country.
    pub name: String,
    pub port: u16,
    /// Most players at once.
    pub capacity: u16,
    /// Shown as "Members" instead of "Free" (world list bit 0), and told to the client when it
    /// enters (`WORLD_MEMBERS`). Nothing here is kept for members only yet, so it only changes
    /// what the client is told.
    pub members: bool,
    /// The Loot share column's icon (world list bit 3). Nothing shares loot yet.
    pub lootshare: bool,
    /// A highlighted world (world list bit 4; the reference server's name for it): the World
    /// Select draws the themed-world icon and the name instead of the flag.
    pub highlighted: bool,
    /// The world is in the list but the World Select doesn't show it (world list bit 12).
    pub hidden: bool,
    /// A quick chat world: world list bit 1, and what the client is told when it enters
    /// (`WORLD_QUICKCHAT`). Typing isn't ported either way, so nothing here enforces it.
    pub quickchat: bool,
    /// World list bit 2 (the reference server's `botsAllowed`). Nothing in this cache's World
    /// Select draws it; it is here because the bit is a world's to set.
    pub bots_allowed: bool,
    /// Which flag shows (enum 1810 of the cache turns it into the sprite), when the world has
    /// no name. Estimated: the numbers are the cache's, and 0 is the one this server starts with.
    pub country: i32,
    /// The location's name, beside the flag.
    pub location: String,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            name: String::new(),
            port: 0,
            capacity: 500,
            members: true,
            lootshare: false,
            highlighted: false,
            hidden: false,
            quickchat: false,
            bots_allowed: false,
            country: 0,
            location: "Local".into(),
        }
    }
}

/// A name the World Select can show: plain ASCII (the list's strings are Windows-1252, where
/// ASCII is the same) and short enough for its column.
fn sayable(text: &str, what: &str) -> Result<(), String> {
    if text.chars().count() > NAME_LEN {
        return Err(format!("the {what} can have {NAME_LEN} characters at most"));
    }
    if !text.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
        return Err(format!("the {what} can only have plain letters, digits, spaces and punctuation"));
    }
    Ok(())
}

impl Options {
    /// Why these can't be used, if they can't.
    pub fn check(&self) -> Result<(), String> {
        sayable(&self.name, "name")?;
        sayable(&self.location, "location")?;
        if self.capacity == 0 || self.capacity > MAX_PLAYERS {
            return Err(format!("a world holds 1 to {MAX_PLAYERS} players"));
        }
        if self.port == 0 {
            return Err("the port can't be 0".into());
        }
        if self.country < 0 {
            return Err("the flag can't be a negative number".into());
        }
        Ok(())
    }

    /// The world list's flags for these options.
    pub fn flags(&self) -> i32 {
        let bit = |on: bool, flag: i32| if on { flag } else { 0 };
        bit(self.members, MEMBERS)
            | bit(self.quickchat, QUICK_CHAT)
            | bit(self.bots_allowed, BOTS_ALLOWED)
            | bit(self.lootshare, LOOTSHARE)
            | bit(self.highlighted, HIGHLIGHTED)
            | bit(self.hidden, HIDDEN)
    }

    /// What the login reply tells a player entering this world.
    pub fn world_info(&self) -> WorldInfo {
        WorldInfo { members: self.members, quickchat_only: self.quickchat }
    }

    /// The world list's activity text: "-" when the world has no name (the World Select reads
    /// a text of one character as "none" and shows the flag instead).
    pub fn activity(&self) -> String {
        if self.name.is_empty() {
            "-".into()
        } else {
            self.name.clone()
        }
    }
}

/// A running world with its number and what it was opened with.
pub struct Hosted {
    /// Its number in the world list: 1 for the first, then 2, 3… in the order they were opened.
    pub number: i32,
    pub options: Options,
    pub world: World,
}

pub struct Worlds {
    hosted: RwLock<Vec<Arc<Hosted>>>,
    /// Map keys by region, the same for every world.
    map_keys: Arc<HashMap<u32, Key>>,
    stats: SharedStats,
    file: Option<PathBuf>,
}

impl Worlds {
    /// No worlds yet; `file` is where the administrator's worlds are saved (`None` = nowhere).
    pub fn new(map_keys: HashMap<u32, Key>, stats: SharedStats, file: Option<PathBuf>) -> Arc<Self> {
        Arc::new(Worlds { hosted: RwLock::default(), map_keys: Arc::new(map_keys), stats, file })
    }

    /// Starts a world with the next number. The caller has checked `options` and holds its port.
    pub fn open(&self, options: Options) -> Arc<Hosted> {
        let mut hosted = self.hosted.write().unwrap();
        let number = hosted.len() as i32 + 1;
        let world = World::start(number, options.capacity, Arc::clone(&self.map_keys), Arc::clone(&self.stats));
        let new = Arc::new(Hosted { number, options, world });
        hosted.push(Arc::clone(&new));
        new
    }

    /// Every world, in number order.
    pub fn all(&self) -> Vec<Arc<Hosted>> {
        self.hosted.read().unwrap().clone()
    }

    /// The administrator's worlds as they were saved, leaving out any that can't be used.
    pub fn saved(&self) -> Vec<Options> {
        let text = self.file.as_ref().and_then(|f| std::fs::read_to_string(f).ok()).unwrap_or_default();
        let saved: Vec<Options> = serde_json::from_str(&text).unwrap_or_default();
        saved.into_iter().filter(|o| o.check().is_ok()).collect()
    }

    /// Saves every world but the first (which `server.cfg` describes).
    pub fn save(&self) -> Result<(), String> {
        let Some(file) = &self.file else { return Ok(()) };
        let worlds: Vec<Options> = self.all().iter().skip(1).map(|h| h.options.clone()).collect();
        let text = serde_json::to_string_pretty(&worlds).map_err(|e| e.to_string())?;
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        std::fs::write(file, text + "\n").map_err(|e| format!("couldn't save the worlds to {}: {e}", file.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(name: &str, port: u16) -> Options {
        Options { name: name.into(), port, ..Options::default() }
    }

    #[test]
    fn worlds_are_numbered_in_the_order_they_open() {
        let worlds = Worlds::new(HashMap::new(), SharedStats::default(), None);
        assert_eq!(worlds.open(options("", 5555)).number, 1);
        assert_eq!(worlds.open(options("Trade", 5556)).number, 2);
        let all = worlds.all();
        assert_eq!((all.len(), all[1].options.name.as_str(), all[1].world.seats().capacity()), (2, "Trade", 500));
        assert_eq!(worlds.save(), Ok(()));
    }

    /// Saving and reading back keeps every option, and a world that can't be used is left out.
    #[test]
    fn saved_worlds_read_back_and_bad_ones_are_left_out() {
        let file = std::env::temp_dir().join("rune-rust-worlds-saved.json");
        let worlds = Worlds::new(HashMap::new(), SharedStats::default(), Some(file));
        worlds.open(options("", 5555));
        let trade = Options {
            name: "Trade and chat".into(),
            port: 5556,
            capacity: 400,
            members: false,
            lootshare: true,
            highlighted: true,
            hidden: true,
            quickchat: true,
            bots_allowed: true,
            country: 22,
            location: "United Kingdom".into(),
        };
        worlds.open(trade.clone());
        worlds.save().unwrap();
        assert_eq!(worlds.saved(), vec![trade]);
        // A saved world with no room for players is left out rather than opened.
        let file = worlds.file.clone().unwrap();
        std::fs::write(&file, r#"[{"port":5556,"capacity":0}]"#).unwrap();
        assert_eq!(worlds.saved(), Vec::new());
        // Fields that aren't in the file keep their default.
        std::fs::write(&file, r#"[{"port":5556,"name":"Old"}]"#).unwrap();
        assert_eq!(worlds.saved(), vec![Options { name: "Old".into(), port: 5556, ..Options::default() }]);
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn options_that_cannot_work_are_refused() {
        assert!(options("ok", 5556).check().is_ok());
        assert!(options(&"x".repeat(NAME_LEN + 1), 5556).check().is_err());
        assert!(options("ação", 5556).check().is_err());
        assert!(options("ok", 0).check().is_err());
        assert!(Options { capacity: MAX_PLAYERS + 1, ..options("ok", 5556) }.check().is_err());
        assert!(Options { location: "ação".into(), ..options("ok", 5556) }.check().is_err());
        assert!(Options { country: -1, ..options("ok", 5556) }.check().is_err());
    }

    /// Each option lands in the bit or the field the client reads it from.
    #[test]
    fn options_become_the_flags_and_the_login_reply() {
        let plain = Options { members: false, ..options("", 1) };
        assert_eq!((plain.flags(), plain.activity().as_str()), (0, "-"));
        assert_eq!(plain.world_info(), WorldInfo { members: false, quickchat_only: false });
        let all = Options { lootshare: true, highlighted: true, hidden: true, quickchat: true, bots_allowed: true, ..options("Trade", 1) };
        assert_eq!(all.flags(), MEMBERS | QUICK_CHAT | BOTS_ALLOWED | LOOTSHARE | HIGHLIGHTED | HIDDEN);
        assert_eq!((all.activity().as_str(), all.world_info().quickchat_only), ("Trade", true));
    }
}
