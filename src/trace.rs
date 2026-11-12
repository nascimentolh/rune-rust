//! Byte-level trace of every connection, written to a log file, so what a real
//! client sends can be read back and the protocol corrected from it.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Bytes shown per read/write; the rest is only counted (cache archives are big).
const SHOWN_BYTES: usize = 96;

#[derive(Clone)]
pub struct TraceFile(Arc<Mutex<File>>);

impl TraceFile {
    pub fn open(path: &Path) -> io::Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        Ok(TraceFile(Arc::new(Mutex::new(OpenOptions::new().create(true).append(true).open(path)?))))
    }

    pub fn note(&self, peer: &str, text: &str) {
        let _ = writeln!(self.0.lock().unwrap(), "{} {peer} {text}", timestamp());
    }

    fn bytes(&self, peer: &str, arrow: &str, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        let shown: Vec<String> = data.iter().take(SHOWN_BYTES).map(|b| format!("{b:02x}")).collect();
        let more = if data.len() > SHOWN_BYTES { format!(" (+{} more)", data.len() - SHOWN_BYTES) } else { String::new() };
        self.note(peer, &format!("{arrow} {} bytes: {}{more}", data.len(), shown.join(" ")));
    }
}

fn timestamp() -> String {
    let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis());
    format!("{}.{:03}", ms / 1000, ms % 1000)
}

/// A stream that copies everything read ("<-", from the client) and written ("->") to the trace.
pub struct Traced<S> {
    inner: S,
    trace: Option<TraceFile>,
    peer: String,
}

impl<S> Traced<S> {
    pub fn new(inner: S, trace: Option<TraceFile>, peer: &str) -> Self {
        Traced { inner, trace, peer: peer.to_string() }
    }
}

impl<S: Read> Read for Traced<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        if let Some(t) = &self.trace {
            t.bytes(&self.peer, "<-", &buf[..n]);
        }
        Ok(n)
    }
}

impl<S: Write> Write for Traced<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        if let Some(t) = &self.trace {
            t.bytes(&self.peer, "->", &buf[..n]);
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
