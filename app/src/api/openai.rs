//! Clients talk to llama-server's OpenAI routes directly.
//!
//! WinServeAI does not re-implement chat completions — it ensures the process
//! is up and documents the base URL: `{host}:{port}/v1`.

use crate::server::config::Config;

pub fn chat_completions_url(config: &Config) -> String {
    format!("{}/chat/completions", config.openai_v1_url())
}

pub fn models_url(config: &Config) -> String {
    format!("{}/models", config.openai_v1_url())
}
