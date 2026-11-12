//! The client's start-up downloads, in the reference client's order
//! (`IndexLoaders.initializeGame`), reported as its loading stages:
//!
//! | stage | reference                                   | here                                        |
//! |-------|---------------------------------------------|---------------------------------------------|
//! | 0     | start the JS5 connection                    | connect + cache handshake                   |
//! | 1     | load indexes 33, 32 (or 34) and 13          | their reference tables and every archive    |
//! | 2–4   | fonts and the first loading screen          | hand the files to the loading screen        |
//! | 5–6   | every other index's reference table (5→98%) | the remaining reference tables              |
//! | 7–8   | defaults file 4, native libraries           | — (passes straight through)                 |
//! | 9     | archives needed before login (0→92%)        | every archive of index 2 (configs), checked |
//! | 10–17 | definition loaders, sprites, maps           | — (passes straight through), then login     |
//!
//! Every download is checked by CRC; runs in a background thread so the screen keeps drawing.

use super::loading_stage::{Status, DONE};
use crate::cache::{archive, container, ReferenceTable};
use crate::protocol::checksum_table;
use crate::protocol::js5 as protocol;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// How long to keep trying while the server process starts up.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(60);
const CONNECT_RETRY: Duration = Duration::from_millis(500);
/// Loading-screen definitions, their images and fonts, and font metrics.
const TIPS_INDEX: u8 = 33;
const LOADING_IMAGES_INDEX: u8 = 32;
const FONT_METRICS_INDEX: u8 = 13;
/// Indexes downloaded in full before login (2 = configs). The reference loads more here
/// (`FriendStatus.method7702`); which archives exactly isn't ported yet.
pub const PRELOAD_INDEXES: [u8; 1] = [2];

#[derive(Clone, Debug)]
pub struct Summary {
    pub indexes: usize,
    pub archives: usize,
    pub files: usize,
    pub bytes: usize,
}

/// The loading-screen files, first file of each archive, by archive id.
#[derive(Default)]
pub struct LoadingAssets {
    pub tips: HashMap<i32, Vec<u8>>,
    pub images: HashMap<i32, Vec<u8>>,
    pub metrics: HashMap<i32, Vec<u8>>,
}

pub struct Progress {
    pub status: Status,
    pub assets: Option<Arc<LoadingAssets>>,
    pub result: Option<Result<Summary, String>>,
    /// The open cache connection, handed to the client when loading is done.
    pub session: Option<Session>,
}

pub type Shared = Arc<Mutex<Progress>>;

pub struct Target {
    pub address: String,
    pub build: i32,
    pub sub_build: i32,
    pub js5_key: String,
    /// How many grab keys follow a good handshake.
    pub grab_key_count: usize,
}

pub fn start(target: Target) -> Shared {
    let shared: Shared = Arc::new(Mutex::new(Progress { status: Status::new(), assets: None, result: None, session: None }));
    let out = Arc::clone(&shared);
    thread::spawn(move || {
        let result = run(&target, &out);
        let mut p = out.lock().unwrap();
        match result {
            Ok((summary, session)) => {
                p.result = Some(Ok(summary));
                p.session = Some(session);
            }
            Err(e) => p.result = Some(Err(e)),
        }
    });
    shared
}

fn report(shared: &Shared, stage: usize, progress: i32) {
    shared.lock().unwrap().status.update(stage, progress);
}

fn connect(address: &str) -> Result<TcpStream, String> {
    let started = Instant::now();
    loop {
        match TcpStream::connect(address) {
            Ok(s) => return Ok(s),
            Err(e) if started.elapsed() > CONNECT_TIMEOUT => return Err(format!("can't reach the server at {address}: {e}")),
            Err(_) => thread::sleep(CONNECT_RETRY),
        }
    }
}

/// A JS5 connection with every reference table loaded; after start-up the client keeps it
/// to fetch archives on demand, as the reference's `JS5Manager` does.
pub struct Session {
    stream: TcpStream,
    tables: Vec<Option<ReferenceTable>>,
    rows: Vec<(i32, i32)>,
    bytes: usize,
}

impl Session {
    fn fetch(&mut self, index: u8, archive: u32) -> Result<Vec<u8>, String> {
        let s = &mut self.stream;
        s.write_all(&protocol::encode_request(index, archive, true)).map_err(|e| e.to_string())?;
        let (i, a, container) = protocol::read_response(s).map_err(|e| format!("{index}/{archive}: {e}"))?;
        if (i, a) != (index, archive) {
            return Err(format!("asked for {index}/{archive}, got {i}/{a}"));
        }
        self.bytes += container.len();
        Ok(container)
    }

    /// Fetches and checks one index's reference table.
    fn load_table(&mut self, index: usize) -> Result<(), String> {
        let crc = self.rows.get(index).map_or(0, |r| r.0);
        if crc == 0 || self.tables[index].is_some() {
            return Ok(());
        }
        let raw = self.fetch(255, index as u32)?;
        if crc32fast::hash(&raw) as i32 != crc {
            return Err(format!("reference table {index} doesn't match its checksum"));
        }
        let table = ReferenceTable::decode(&container::decompress(&raw).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        self.tables[index] = Some(table);
        Ok(())
    }

    /// Fetches, checks and unpacks one archive; returns its files.
    fn load_archive(&mut self, index: u8, archive: u32) -> Result<Vec<Vec<u8>>, String> {
        let entry = self.tables[index as usize]
            .as_ref()
            .and_then(|t| t.archives.iter().find(|a| a.id == archive))
            .ok_or(format!("archive {index}/{archive} isn't listed"))?;
        let (crc, file_count) = (entry.crc, entry.files.len());
        let raw = self.fetch(index, archive)?;
        if crc32fast::hash(&raw) as i32 != crc {
            return Err(format!("archive {index}/{archive} doesn't match its checksum"));
        }
        let data = container::decompress(&raw).map_err(|e| format!("archive {index}/{archive}: {e}"))?;
        archive::split(&data, file_count).map_err(|e| format!("archive {index}/{archive}: {e}"))
    }

    fn archive_ids(&self, index: u8) -> Vec<u32> {
        self.tables.get(index as usize).and_then(|t| t.as_ref()).map_or(Vec::new(), |t| t.archives.iter().map(|a| a.id).collect())
    }
}

impl crate::client::source::Source for Session {
    fn files(&mut self, index: u8, archive: u32) -> Option<crate::client::source::Files> {
        let table = self.tables.get(index as usize)?.as_ref()?;
        let ids: Vec<u32> = table.archives.iter().find(|a| a.id == archive)?.files.iter().map(|f| f.id).collect();
        Some(crate::client::source::by_id(&ids, self.load_archive(index, archive).ok()?))
    }
}

fn run(target: &Target, shared: &Shared) -> Result<(Summary, Session), String> {
    // Stage 0: connect and shake hands.
    let mut stream = connect(&target.address)?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    stream.write_all(&protocol::encode_handshake(target.build, target.sub_build, &target.js5_key)).map_err(|e| e.to_string())?;
    let mut reply = [0u8; 1];
    stream.read_exact(&mut reply).map_err(|e| e.to_string())?;
    if reply[0] != protocol::STATUS_OK {
        return Err(format!("the server refused this client version (code {})", reply[0]));
    }
    let mut grab_keys = vec![0u8; target.grab_key_count * 4];
    stream.read_exact(&mut grab_keys).map_err(|e| e.to_string())?;
    let mut s = Session { stream, tables: Vec::new(), rows: Vec::new(), bytes: 0 };
    s.rows = checksum_table::parse(&s.fetch(255, 255)?).map_err(|e| e.to_string())?;
    s.tables = vec![None; s.rows.len()];

    // Stage 1: the loading-screen indexes, in full.
    report(shared, 1, 0);
    let mut assets = LoadingAssets::default();
    let first = [TIPS_INDEX, LOADING_IMAGES_INDEX, FONT_METRICS_INDEX];
    for index in first {
        s.load_table(index as usize)?;
    }
    let wanted: Vec<(u8, u32)> = first.iter().flat_map(|&i| s.archive_ids(i).into_iter().map(move |a| (i, a))).collect();
    for (n, &(index, id)) in wanted.iter().enumerate() {
        report(shared, 1, (n * 100 / wanted.len().max(1)) as i32);
        let file = s.load_archive(index, id)?.into_iter().next().unwrap_or_default();
        let map = match index {
            TIPS_INDEX => &mut assets.tips,
            LOADING_IMAGES_INDEX => &mut assets.images,
            _ => &mut assets.metrics,
        };
        map.insert(id as i32, file);
    }

    // Stages 2–4: fonts and loading screens are ready.
    for stage in 2..=4 {
        report(shared, stage, 100);
    }
    shared.lock().unwrap().assets = Some(Arc::new(assets));

    // Stages 5–6: every other reference table.
    report(shared, 5, 100);
    let total = s.rows.len();
    for index in 0..total {
        report(shared, 6, (index * 100 / total.max(1)) as i32);
        s.load_table(index)?;
    }
    report(shared, 6, 100);
    report(shared, 7, 100);
    report(shared, 8, 100);

    // Stage 9: archives needed before login.
    let preload: Vec<(u8, u32)> = PRELOAD_INDEXES.iter().flat_map(|&i| s.archive_ids(i).into_iter().map(move |a| (i, a))).collect();
    let mut files = 0;
    for (n, &(index, id)) in preload.iter().enumerate() {
        report(shared, 9, (n * 100 / preload.len().max(1)) as i32);
        files += s.load_archive(index, id)?.len();
    }
    for stage in 9..=DONE {
        report(shared, stage, 100);
    }
    let summary = Summary { indexes: s.tables.iter().flatten().count(), archives: preload.len(), files, bytes: s.bytes };
    Ok((summary, s))
}
