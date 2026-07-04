//! Port / bind helpers.

use std::net::TcpListener;

/// True if `host:port` can be bound (port free enough for our purposes).
pub fn port_available(host: &str, port: u16) -> bool {
    let addr = format!("{host}:{port}");
    TcpListener::bind(addr).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ephemeral_port_is_available() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        assert!(port_available("127.0.0.1", port));
    }

    #[test]
    fn bound_port_is_unavailable() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(!port_available("127.0.0.1", port));
    }
}
