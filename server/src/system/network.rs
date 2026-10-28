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
        // Port may briefly linger or be raced by parallel tests — retry.
        let ok = (0..20).any(|_| {
            if port_available("127.0.0.1", port) {
                true
            } else {
                std::thread::sleep(std::time::Duration::from_millis(5));
                false
            }
        });
        assert!(ok, "port {port} still unavailable after release");
    }

    #[test]
    fn bound_port_is_unavailable() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(!port_available("127.0.0.1", port));
    }
}
