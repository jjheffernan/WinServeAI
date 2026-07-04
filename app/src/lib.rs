//! WinServeAI — Windows-native llama.cpp appliance wrapper.
//!
//! Runtime boundary:
//!
//! ```text
//! Windows App / CLI
//!        │
//!        ▼
//!   ServerManager   (app/server)
//!        │
//!        ▼
//!   runtime/llama   builds command, owns process
//!        │
//!        ▼
//!   bin/llama-server.exe   (external)
//!        │
//!        ▼
//!   OpenAI API on host:port
//! ```
//!
//! One backend only. No plugin system. No backend trait.

pub mod api;
pub mod runtime;
pub mod server;
pub mod system;

pub use server::manager::ServerManager;
pub use server::config::Config;
