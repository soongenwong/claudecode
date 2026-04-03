use std::future::Future;
use std::pin::Pin;

use crate::error::ApiError;
use crate::types::{MessageRequest, MessageResponse};

pub mod claw_provider;
pub mod github_copilot;
pub mod openai_compat;

pub type ProviderFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ApiError>> + Send + 'a>>;

pub trait Provider {
    type Stream;

    fn send_message<'a>(
        &'a self,
        request: &'a MessageRequest,
    ) -> ProviderFuture<'a, MessageResponse>;

    fn stream_message<'a>(
        &'a self,
        request: &'a MessageRequest,
    ) -> ProviderFuture<'a, Self::Stream>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    ClawApi,
    GithubCopilot,
    Xai,
    OpenAi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderMetadata {
    pub provider: ProviderKind,
    pub auth_env: &'static str,
    pub base_url_env: &'static str,
    pub default_base_url: &'static str,
}

const MODEL_REGISTRY: &[(&str, ProviderMetadata)] = &[
    (
        "opus",
        ProviderMetadata {
            provider: ProviderKind::ClawApi,
            auth_env: "ANTHROPIC_API_KEY",
            base_url_env: "ANTHROPIC_BASE_URL",
            default_base_url: claw_provider::DEFAULT_BASE_URL,
        },
    ),
    (
        "sonnet",
        ProviderMetadata {
            provider: ProviderKind::ClawApi,
            auth_env: "ANTHROPIC_API_KEY",
            base_url_env: "ANTHROPIC_BASE_URL",
            default_base_url: claw_provider::DEFAULT_BASE_URL,
        },
    ),
    (
        "haiku",
        ProviderMetadata {
            provider: ProviderKind::ClawApi,
            auth_env: "ANTHROPIC_API_KEY",
            base_url_env: "ANTHROPIC_BASE_URL",
            default_base_url: claw_provider::DEFAULT_BASE_URL,
        },
    ),
    (
        "claude-opus-4-6",
        ProviderMetadata {
            provider: ProviderKind::ClawApi,
            auth_env: "ANTHROPIC_API_KEY",
            base_url_env: "ANTHROPIC_BASE_URL",
            default_base_url: claw_provider::DEFAULT_BASE_URL,
        },
    ),
    (
        "claude-sonnet-4-6",
        ProviderMetadata {
            provider: ProviderKind::ClawApi,
            auth_env: "ANTHROPIC_API_KEY",
            base_url_env: "ANTHROPIC_BASE_URL",
            default_base_url: claw_provider::DEFAULT_BASE_URL,
        },
    ),
    (
        "claude-haiku-4-5-20251213",
        ProviderMetadata {
            provider: ProviderKind::ClawApi,
            auth_env: "ANTHROPIC_API_KEY",
            base_url_env: "ANTHROPIC_BASE_URL",
            default_base_url: claw_provider::DEFAULT_BASE_URL,
        },
    ),
    (
        "github-copilot/gpt-4o",
        ProviderMetadata {
            provider: ProviderKind::GithubCopilot,
            auth_env: "COPILOT_GITHUB_TOKEN",
            base_url_env: "COPILOT_BASE_URL",
            default_base_url: github_copilot::DEFAULT_BASE_URL,
        },
    ),
    (
        "github-copilot/gpt-5.4",
        ProviderMetadata {
            provider: ProviderKind::GithubCopilot,
            auth_env: "COPILOT_GITHUB_TOKEN",
            base_url_env: "COPILOT_BASE_URL",
            default_base_url: github_copilot::DEFAULT_BASE_URL,
        },
    ),
    (
        "grok",
        ProviderMetadata {
            provider: ProviderKind::Xai,
            auth_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_XAI_BASE_URL,
        },
    ),
    (
        "grok-3",
        ProviderMetadata {
            provider: ProviderKind::Xai,
            auth_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_XAI_BASE_URL,
        },
    ),
    (
        "grok-mini",
        ProviderMetadata {
            provider: ProviderKind::Xai,
            auth_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_XAI_BASE_URL,
        },
    ),
    (
        "grok-3-mini",
        ProviderMetadata {
            provider: ProviderKind::Xai,
            auth_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_XAI_BASE_URL,
        },
    ),
    (
        "grok-2",
        ProviderMetadata {
            provider: ProviderKind::Xai,
            auth_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_XAI_BASE_URL,
        },
    ),
];

#[must_use]
pub fn resolve_model_alias(model: &str) -> String {
    let trimmed = model.trim();
    let lower = trimmed.to_ascii_lowercase();
    MODEL_REGISTRY
        .iter()
        .find_map(|(alias, metadata)| {
            (*alias == lower).then_some(match metadata.provider {
                ProviderKind::ClawApi => match *alias {
                    "opus" => "claude-opus-4-6",
                    "sonnet" => "claude-sonnet-4-6",
                    "haiku" => "claude-haiku-4-5-20251213",
                    _ => trimmed,
                },
                ProviderKind::Xai => match *alias {
                    "grok" | "grok-3" => "grok-3",
                    "grok-mini" | "grok-3-mini" => "grok-3-mini",
                    "grok-2" => "grok-2",
                    _ => trimmed,
                },
                ProviderKind::GithubCopilot => trimmed,
                ProviderKind::OpenAi => trimmed,
            })
        })
        .map_or_else(|| trimmed.to_string(), ToOwned::to_owned)
}

#[must_use]
pub fn metadata_for_model(model: &str) -> Option<ProviderMetadata> {
    let canonical = resolve_model_alias(model);
    let lower = canonical.to_ascii_lowercase();
    if let Some((_, metadata)) = MODEL_REGISTRY.iter().find(|(alias, _)| *alias == lower) {
        return Some(*metadata);
    }
    if lower.starts_with("github-copilot/") {
        return Some(ProviderMetadata {
            provider: ProviderKind::GithubCopilot,
            auth_env: "COPILOT_GITHUB_TOKEN",
            base_url_env: "COPILOT_BASE_URL",
            default_base_url: github_copilot::DEFAULT_BASE_URL,
        });
    }
    if lower.starts_with("grok") {
        return Some(ProviderMetadata {
            provider: ProviderKind::Xai,
            auth_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_XAI_BASE_URL,
        });
    }
    None
}

#[must_use]
pub fn detect_provider_kind(model: &str) -> ProviderKind {
    if let Some(metadata) = metadata_for_model(model) {
        return metadata.provider;
    }
    if claw_provider::has_auth_from_env_or_saved().unwrap_or(false) {
        return ProviderKind::ClawApi;
    }
    if openai_compat::has_api_key("OPENAI_API_KEY") {
        return ProviderKind::OpenAi;
    }
    if openai_compat::has_api_key("XAI_API_KEY") {
        return ProviderKind::Xai;
    }
    if github_copilot::load_saved_github_token()
        .ok()
        .and_then(std::convert::identity)
        .is_some()
        || github_copilot::load_cached_runtime_token()
            .ok()
            .and_then(std::convert::identity)
            .is_some()
    {
        return ProviderKind::GithubCopilot;
    }
    ProviderKind::ClawApi
}

#[must_use]
pub fn max_tokens_for_model(model: &str) -> u32 {
    let canonical = resolve_model_alias(model);
    if canonical.starts_with("github-copilot/") {
        8_192
    } else if canonical.contains("opus") {
        32_000
    } else {
        64_000
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_provider_kind, max_tokens_for_model, resolve_model_alias, ProviderKind};

    #[test]
    fn resolves_grok_aliases() {
        assert_eq!(resolve_model_alias("grok"), "grok-3");
        assert_eq!(resolve_model_alias("grok-mini"), "grok-3-mini");
        assert_eq!(resolve_model_alias("grok-2"), "grok-2");
    }

    #[test]
    fn detects_provider_from_model_name_first() {
        assert_eq!(detect_provider_kind("grok"), ProviderKind::Xai);
        assert_eq!(
            detect_provider_kind("github-copilot/gpt-4o"),
            ProviderKind::GithubCopilot
        );
        assert_eq!(
            detect_provider_kind("claude-sonnet-4-6"),
            ProviderKind::ClawApi
        );
    }

    #[test]
    fn keeps_existing_max_token_heuristic() {
        assert_eq!(max_tokens_for_model("opus"), 32_000);
        assert_eq!(max_tokens_for_model("grok-3"), 64_000);
        assert_eq!(max_tokens_for_model("github-copilot/gpt-4o"), 8_192);
    }
}
