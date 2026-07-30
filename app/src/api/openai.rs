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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::config::Config;

    #[test]
    fn urls_append_openai_routes() {
        let cfg = Config::default();
        assert_eq!(
            models_url(&cfg),
            format!("{}/models", cfg.openai_v1_url())
        );
        assert_eq!(
            chat_completions_url(&cfg),
            format!("{}/chat/completions", cfg.openai_v1_url())
        );
        assert!(models_url(&cfg).ends_with("/v1/models"));
        assert!(chat_completions_url(&cfg).ends_with("/v1/chat/completions"));
    }
}
