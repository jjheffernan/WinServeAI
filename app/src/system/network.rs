//! Port / bind helpers.

use std::net::TcpListener;

/// True if `host:port` can be bound (port free enough for our purposes).
pub fn port_available(host: &str, port: u16) -> bool {
    let addr = format!("{host}:{port}");
    TcpListener::bind(addr).is_ok()
}
