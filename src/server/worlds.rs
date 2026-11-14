//! The game worlds this server runs: the first one, made from `server.cfg`, and any the
//! administrator opens from the status window. Each has its own port, its own thread and its
//! own players (`super::world`); nothing is shared between two worlds.
//!
//! There is no reference server, so what a world can be given is this server's choice, kept to
//! what the world list can carry (`crate::protocol::world_list`): the text of its activity
//! column, used here as its name, and the two flags the client knows.
//!
//! Worlds opened by the administrator are saved one per line as
//! `port capacity members lootshare name` and opened again when the server starts. To remove
//! one, delete its line while the server is stopped.

use super::net::SharedStats;
use super::world::{World, MAX_PLAYERS};
use crate::cache::xtea::Key;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// The longest name a world may have. This server's choice: what fits the list's column.
pub const NAME_LEN: usize = 24;

/// What the administrator chooses for a world.
#[derive(Clone, Debug, PartialEq)]
pub struct Options {
    /// Shown in the world list's activity column; empty for none.
    pub name: String,
    pub port: u16,
    /// Most players at once.
    pub capacity: u16,
    /// The world list's members flag and LootShare flag.
    pub members: bool,
    pub lootshare: bool,
}

impl Options {
    /// Why these can't be used, if they can't.
    pub fn check(&self) -> Result<(), String> {
        if self.name.chars().count() > NAME_LEN {
            return Err(format!("the name can have {NAME_LEN} characters at most"));
        }
        // The world list's strings are Windows-1252; plain ASCII is the same in both.
        if !self.name.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
            return Err("the name can only have plain letters, digits, spaces and punctuation".into());
        }
        if self.capacity == 0 || self.capacity > MAX_PLAYERS {
            return Err(format!("a world holds 1 to {MAX_PLAYERS} players"));
        }
        if self.port == 0 {
            return Err("the port can't be 0".into());
        }
        Ok(())
    }

    fn to_line(&self) -> String {
        format!("{} {} {} {} {}", self.port, self.capacity, self.members, self.lootshare, self.name).trim_end().to_string()
    }

    fn from_line(line: &str) -> Option<Self> {
        let mut parts = line.splitn(5, ' ');
        let options = Options {
            port: parts.next()?.parse().ok()?,
            capacity: parts.next()?.parse().ok()?,
            members: parts.next()?.parse().ok()?,
            lootshare: parts.next()?.parse().ok()?,
            name: parts.next().unwrap_or("").trim().to_string(),
        };
        options.check().ok().map(|()| options)
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

    /// The administrator's worlds as they were saved.
    pub fn saved(&self) -> Vec<Options> {
        let text = self.file.as_ref().and_then(|f| std::fs::read_to_string(f).ok()).unwrap_or_default();
        text.lines().filter_map(Options::from_line).collect()
    }

    /// Saves every world but the first (which `server.cfg` describes).
    pub fn save(&self) -> Result<(), String> {
        let Some(file) = &self.file else { return Ok(()) };
        let lines: Vec<String> = self.all().iter().skip(1).map(|h| h.options.to_line()).collect();
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        std::fs::write(file, lines.join("\n") + "\n").map_err(|e| format!("couldn't save the worlds to {}: {e}", file.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(name: &str, port: u16) -> Options {
        Options { name: name.into(), port, capacity: 500, members: true, lootshare: false }
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

    #[test]
    fn a_saved_line_reads_back_and_a_bad_one_is_left_out() {
        let trade = options("Trade and chat", 5556);
        assert_eq!(trade.to_line(), "5556 500 true false Trade and chat");
        assert_eq!(Options::from_line(&trade.to_line()), Some(trade));
        assert_eq!(Options::from_line(&options("", 5557).to_line()), Some(options("", 5557)));
        assert_eq!(Options::from_line("5556 0 true false no room"), None);
        assert_eq!(Options::from_line("nonsense"), None);
    }

    #[test]
    fn options_that_cannot_work_are_refused() {
        assert!(options("ok", 5556).check().is_ok());
        assert!(options(&"x".repeat(NAME_LEN + 1), 5556).check().is_err());
        assert!(options("ação", 5556).check().is_err());
        assert!(options("ok", 0).check().is_err());
        assert!(Options { capacity: MAX_PLAYERS + 1, ..options("ok", 5556) }.check().is_err());
    }
}
