//! Core library crate.

pub mod config;
pub mod ids;
pub mod models;
pub mod provider;

pub use ids::derive_resume_hash;
pub use provider::{ProviderKind, ProviderParseError};

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{
        ProviderKind, derive_resume_hash,
        config::CoreConfig,
        models::{ConversationLocator, ConversationTranscript, MessageRole, TranscriptMessage},
    };

    #[test]
    fn derive_resume_hash_matches_golden_vectors() {
        let ascii = derive_resume_hash(ProviderKind::Claude, "conversation-123");
        let non_ascii = derive_resume_hash(ProviderKind::Claude, "snowman-\u{2603}");

        assert_eq!(ascii, "93ca551d9459a5b1");
        assert_eq!(non_ascii, "153ce737c3313c82");
        assert_eq!(ascii.len(), 16);
        assert!(ascii.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn provider_kind_round_trips_through_contract_json() {
        let transcript = ConversationTranscript {
            locator: ConversationLocator {
                provider: ProviderKind::Claude,
                conversation_id: "conversation-123".to_owned(),
            },
            messages: vec![
                TranscriptMessage {
                    message_id: "msg-1".to_owned(),
                    role: MessageRole::User,
                    content: "hello".to_owned(),
                },
                TranscriptMessage {
                    message_id: "msg-2".to_owned(),
                    role: MessageRole::Assistant,
                    content: "hi".to_owned(),
                },
            ],
        };

        let value = serde_json::to_value(&transcript).expect("transcript should serialize");
        assert_eq!(
            value,
            json!({
                "locator": {
                    "provider": "claude",
                    "conversation_id": "conversation-123",
                },
                "messages": [
                    {
                        "message_id": "msg-1",
                        "role": "user",
                        "content": "hello",
                    },
                    {
                        "message_id": "msg-2",
                        "role": "assistant",
                        "content": "hi",
                    }
                ],
            })
        );

        let round_trip: ConversationTranscript =
            serde_json::from_value(value).expect("transcript should deserialize");
        assert_eq!(round_trip, transcript);
    }

    #[test]
    fn core_config_round_trips_through_contract_json() {
        let config = CoreConfig {
            default_provider: Some(ProviderKind::Gemini),
        };

        let value = serde_json::to_value(&config).expect("config should serialize");
        assert_eq!(value, json!({ "default_provider": "gemini" }));

        let round_trip: CoreConfig =
            serde_json::from_value(value).expect("config should deserialize");
        assert_eq!(round_trip, config);
    }

    #[test]
    fn provider_kind_parses_claude_zed_and_antigravity_cli_slugs() {
        let claude = ProviderKind::from_slug("claude").expect("claude should parse");
        let zed = ProviderKind::from_slug("zed").expect("zed should parse");
        let antigravity_cli = ProviderKind::from_slug("antigravity-cli")
            .expect("antigravity-cli should parse");

        assert_eq!(claude, ProviderKind::Claude);
        assert_eq!(claude.as_slug(), "claude");
        assert_eq!(zed, ProviderKind::Zed);
        assert_eq!(zed.as_slug(), "zed");
        assert_eq!(antigravity_cli, ProviderKind::AntigravityCli);
        assert_eq!(antigravity_cli.as_slug(), "antigravity-cli");
    }

    #[test]
    fn provider_kind_reports_typed_parse_errors() {
        let error = ProviderKind::from_slug("unknown-provider")
            .expect_err("unknown slug should return a typed parse error");

        assert_eq!(error.slug(), "unknown-provider");
        assert_eq!(error.to_string(), "unknown provider slug: unknown-provider");
    }
}
