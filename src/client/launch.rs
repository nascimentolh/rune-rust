//! Starting the server as its own process when the client finds none running.

use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::process::{Child, Command};
use std::time::Duration;

const PROBE_TIMEOUT: Duration = Duration::from_millis(300);

fn is_listening(address: &str) -> bool {
    let addrs: Vec<SocketAddr> = address.to_socket_addrs().map(|a| a.collect()).unwrap_or_default();
    addrs.iter().any(|a| TcpStream::connect_timeout(a, PROBE_TIMEOUT).is_ok())
}

/// Returns the started server process, or `None` if one was already running
/// (or it couldn't be started — the loading screen then reports the connection error).
pub fn ensure_server(address: &str) -> Option<Child> {
    if is_listening(address) {
        return None;
    }
    let exe = std::env::current_exe().ok()?;
    Command::new(exe).arg("--server").spawn().ok()
}
