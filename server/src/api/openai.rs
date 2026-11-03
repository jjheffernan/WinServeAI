//! Clients talk to llama-server routes directly.
//!
//! WinServeAI does not re-implement completions — it ensures the process
//! is up and documents base URLs: OpenAI `{host}:{port}/v1`, Anthropic
//! `{host}:{port}` (clients append `/v1/messages`).

use crate::server::config::Config;

pub fn models_url(config: &Config) -> String {
    format!("{}/models", config.openai_v1_url())
}

pub fn chat_completions_url(config: &Config) -> String {
    format!("{}/chat/completions", config.openai_v1_url())
}

pub fn responses_url(config: &Config) -> String {
    format!("{}/responses", config.openai_v1_url())
}

pub fn completions_url(config: &Config) -> String {
    format!("{}/completions", config.openai_v1_url())
}

/// Anthropic Messages on llama-server (pin b9866+).
pub fn anthropic_messages_url(config: &Config) -> String {
    format!("{}/v1/messages", config.base_url())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::config::Config;

    #[test]
    fn urls_cover_common_completion_routes() {
        let cfg = Config::default();
        let v1 = cfg.openai_v1_url();
        assert_eq!(models_url(&cfg), format!("{v1}/models"));
        assert_eq!(chat_completions_url(&cfg), format!("{v1}/chat/completions"));
        assert_eq!(responses_url(&cfg), format!("{v1}/responses"));
        assert_eq!(completions_url(&cfg), format!("{v1}/completions"));
        assert_eq!(
            anthropic_messages_url(&cfg),
            format!("{}/v1/messages", cfg.base_url())
        );
        assert!(!anthropic_messages_url(&cfg).contains("/v1/v1/"));
    }
}
