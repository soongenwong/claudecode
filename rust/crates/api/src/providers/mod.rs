use std::future::Future;
use std::pin::Pin;

use crate::error::ApiError;
use crate::types::{MessageRequest, MessageResponse};

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
    Xai,
    OpenAi,
    OpenRouter,
    HuggingFace,
    GitHubModels,
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
    (
        "fast",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openai_compat::DEFAULT_OPENROUTER_BASE_URL,
        },
    ),
    (
        "openrouter-auto",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openai_compat::DEFAULT_OPENROUTER_BASE_URL,
        },
    ),
    (
        "openrouter",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openai_compat::DEFAULT_OPENROUTER_BASE_URL,
        },
    ),
    (
        "openrouter/auto",
        ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openai_compat::DEFAULT_OPENROUTER_BASE_URL,
        },
    ),
    (
        "code",
        ProviderMetadata {
            provider: ProviderKind::HuggingFace,
            auth_env: "HF_TOKEN",
            base_url_env: "HUGGINGFACE_BASE_URL",
            default_base_url: openai_compat::DEFAULT_HUGGING_FACE_BASE_URL,
        },
    ),
    (
        "coder",
        ProviderMetadata {
            provider: ProviderKind::HuggingFace,
            auth_env: "HF_TOKEN",
            base_url_env: "HUGGINGFACE_BASE_URL",
            default_base_url: openai_compat::DEFAULT_HUGGING_FACE_BASE_URL,
        },
    ),
    (
        "hf",
        ProviderMetadata {
            provider: ProviderKind::HuggingFace,
            auth_env: "HF_TOKEN",
            base_url_env: "HUGGINGFACE_BASE_URL",
            default_base_url: openai_compat::DEFAULT_HUGGING_FACE_BASE_URL,
        },
    ),
    (
        "huggingface",
        ProviderMetadata {
            provider: ProviderKind::HuggingFace,
            auth_env: "HF_TOKEN",
            base_url_env: "HUGGINGFACE_BASE_URL",
            default_base_url: openai_compat::DEFAULT_HUGGING_FACE_BASE_URL,
        },
    ),
    (
        "qwen/qwen3-coder-next:fastest",
        ProviderMetadata {
            provider: ProviderKind::HuggingFace,
            auth_env: "HF_TOKEN",
            base_url_env: "HUGGINGFACE_BASE_URL",
            default_base_url: openai_compat::DEFAULT_HUGGING_FACE_BASE_URL,
        },
    ),
    (
        "reasoner",
        ProviderMetadata {
            provider: ProviderKind::HuggingFace,
            auth_env: "HF_TOKEN",
            base_url_env: "HUGGINGFACE_BASE_URL",
            default_base_url: openai_compat::DEFAULT_HUGGING_FACE_BASE_URL,
        },
    ),
    (
        "assistant",
        ProviderMetadata {
            provider: ProviderKind::HuggingFace,
            auth_env: "HF_TOKEN",
            base_url_env: "HUGGINGFACE_BASE_URL",
            default_base_url: openai_compat::DEFAULT_HUGGING_FACE_BASE_URL,
        },
    ),
    (
        "hf-reasoner",
        ProviderMetadata {
            provider: ProviderKind::HuggingFace,
            auth_env: "HF_TOKEN",
            base_url_env: "HUGGINGFACE_BASE_URL",
            default_base_url: openai_compat::DEFAULT_HUGGING_FACE_BASE_URL,
        },
    ),
    (
        "deepseek-ai/deepseek-r1:fastest",
        ProviderMetadata {
            provider: ProviderKind::HuggingFace,
            auth_env: "HF_TOKEN",
            base_url_env: "HUGGINGFACE_BASE_URL",
            default_base_url: openai_compat::DEFAULT_HUGGING_FACE_BASE_URL,
        },
    ),
    (
        "github",
        ProviderMetadata {
            provider: ProviderKind::GitHubModels,
            auth_env: "GITHUB_TOKEN",
            base_url_env: "GITHUB_MODELS_BASE_URL",
            default_base_url: openai_compat::DEFAULT_GITHUB_MODELS_BASE_URL,
        },
    ),
    (
        "github-fast",
        ProviderMetadata {
            provider: ProviderKind::GitHubModels,
            auth_env: "GITHUB_TOKEN",
            base_url_env: "GITHUB_MODELS_BASE_URL",
            default_base_url: openai_compat::DEFAULT_GITHUB_MODELS_BASE_URL,
        },
    ),
    (
        "github-balanced",
        ProviderMetadata {
            provider: ProviderKind::GitHubModels,
            auth_env: "GITHUB_TOKEN",
            base_url_env: "GITHUB_MODELS_BASE_URL",
            default_base_url: openai_compat::DEFAULT_GITHUB_MODELS_BASE_URL,
        },
    ),
    (
        "openai/gpt-4.1-mini",
        ProviderMetadata {
            provider: ProviderKind::GitHubModels,
            auth_env: "GITHUB_TOKEN",
            base_url_env: "GITHUB_MODELS_BASE_URL",
            default_base_url: openai_compat::DEFAULT_GITHUB_MODELS_BASE_URL,
        },
    ),
    (
        "openai/gpt-4o-mini",
        ProviderMetadata {
            provider: ProviderKind::GitHubModels,
            auth_env: "GITHUB_TOKEN",
            base_url_env: "GITHUB_MODELS_BASE_URL",
            default_base_url: openai_compat::DEFAULT_GITHUB_MODELS_BASE_URL,
        },
    ),
    (
        "deepseek-v3",
        ProviderMetadata {
            provider: ProviderKind::GitHubModels,
            auth_env: "GITHUB_TOKEN",
            base_url_env: "GITHUB_MODELS_BASE_URL",
            default_base_url: openai_compat::DEFAULT_GITHUB_MODELS_BASE_URL,
        },
    ),
    (
        "deepseek-v3-0324",
        ProviderMetadata {
            provider: ProviderKind::GitHubModels,
            auth_env: "GITHUB_TOKEN",
            base_url_env: "GITHUB_MODELS_BASE_URL",
            default_base_url: openai_compat::DEFAULT_GITHUB_MODELS_BASE_URL,
        },
    ),
    (
        "deepseek/deepseek-v3-0324",
        ProviderMetadata {
            provider: ProviderKind::GitHubModels,
            auth_env: "GITHUB_TOKEN",
            base_url_env: "GITHUB_MODELS_BASE_URL",
            default_base_url: openai_compat::DEFAULT_GITHUB_MODELS_BASE_URL,
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
                ProviderKind::Xai => match *alias {
                    "grok" | "grok-3" => "grok-3",
                    "grok-mini" | "grok-3-mini" => "grok-3-mini",
                    "grok-2" => "grok-2",
                    _ => trimmed,
                },
                ProviderKind::OpenAi => trimmed,
                ProviderKind::OpenRouter => match *alias {
                    "fast" | "openrouter" | "openrouter-auto" => "openrouter/auto",
                    _ => trimmed,
                },
                ProviderKind::HuggingFace => match *alias {
                    "code" | "coder" | "hf" | "huggingface" => {
                        "Qwen/Qwen3-Coder-Next:fastest"
                    }
                    "reasoner" | "assistant" | "hf-reasoner" => {
                        "deepseek-ai/DeepSeek-R1:fastest"
                    }
                    _ => trimmed,
                },
                ProviderKind::GitHubModels => match *alias {
                    "github" | "github-balanced" => "openai/gpt-4o-mini",
                    "github-fast" => "openai/gpt-4.1-mini",
                    "deepseek-v3" | "deepseek-v3-0324" => "deepseek/DeepSeek-V3-0324",
                    _ => trimmed,
                },
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
    if lower.starts_with("clues/") {
        return Some(ProviderMetadata {
            provider: ProviderKind::OpenAi,
            auth_env: "OPENAI_API_KEY",
            base_url_env: "OPENAI_BASE_URL",
            default_base_url: openai_compat::DEFAULT_OPENAI_BASE_URL,
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
    if lower == "openrouter/auto" || lower.starts_with("openrouter/") || lower.ends_with(":free") {
        return Some(ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openai_compat::DEFAULT_OPENROUTER_BASE_URL,
        });
    }
    if lower.ends_with(":fastest")
        || lower.ends_with(":cheapest")
        || lower.ends_with(":preferred")
    {
        return Some(ProviderMetadata {
            provider: ProviderKind::HuggingFace,
            auth_env: "HF_TOKEN",
            base_url_env: "HUGGINGFACE_BASE_URL",
            default_base_url: openai_compat::DEFAULT_HUGGING_FACE_BASE_URL,
        });
    }
    if canonical.eq_ignore_ascii_case("deepseek/DeepSeek-V3-0324")
        && openai_compat::has_api_key("GITHUB_TOKEN")
    {
        return Some(ProviderMetadata {
            provider: ProviderKind::GitHubModels,
            auth_env: "GITHUB_TOKEN",
            base_url_env: "GITHUB_MODELS_BASE_URL",
            default_base_url: openai_compat::DEFAULT_GITHUB_MODELS_BASE_URL,
        });
    }
    if lower.contains('/')
        && openai_compat::has_any_api_key(
            openai_compat::OpenAiCompatConfig::openrouter().credential_env_vars(),
        )
    {
        return Some(ProviderMetadata {
            provider: ProviderKind::OpenRouter,
            auth_env: "OPENROUTER_API_KEY",
            base_url_env: "OPENROUTER_BASE_URL",
            default_base_url: openai_compat::DEFAULT_OPENROUTER_BASE_URL,
        });
    }
    if lower.contains('/')
        && openai_compat::has_any_api_key(
            openai_compat::OpenAiCompatConfig::hugging_face().credential_env_vars(),
        )
    {
        return Some(ProviderMetadata {
            provider: ProviderKind::HuggingFace,
            auth_env: "HF_TOKEN",
            base_url_env: "HUGGINGFACE_BASE_URL",
            default_base_url: openai_compat::DEFAULT_HUGGING_FACE_BASE_URL,
        });
    }
    None
}

#[must_use]
pub fn detect_provider_kind(model: &str) -> ProviderKind {
    if let Some(metadata) = metadata_for_model(model) {
        return metadata.provider;
    }
    if openai_compat::has_any_api_key(
        openai_compat::OpenAiCompatConfig::openrouter().credential_env_vars(),
    ) {
        return ProviderKind::OpenRouter;
    }
    if openai_compat::has_any_api_key(
        openai_compat::OpenAiCompatConfig::hugging_face().credential_env_vars(),
    ) {
        return ProviderKind::HuggingFace;
    }
    if openai_compat::has_api_key("GITHUB_TOKEN") {
        return ProviderKind::GitHubModels;
    }
    if openai_compat::has_api_key("XAI_API_KEY") {
        return ProviderKind::Xai;
    }
    if openai_compat::has_api_key("OPENAI_API_KEY") {
        return ProviderKind::OpenAi;
    }
    ProviderKind::OpenRouter
}

#[must_use]
pub fn max_tokens_for_model(model: &str) -> u32 {
    let canonical = resolve_model_alias(model);
    let lower = canonical.to_ascii_lowercase();

    match lower.as_str() {
        "deepseek/deepseek-v3-0324" => 1_024,
        "openai/gpt-4.1-mini" | "openai/gpt-4o-mini" | "meta/llama-3.3-70b-instruct" => 2_048,
        _ if detect_provider_kind(&canonical) == ProviderKind::GitHubModels => 2_048,
        _ => 64_000,
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_provider_kind, max_tokens_for_model, resolve_model_alias, ProviderKind};

    #[test]
    fn resolves_provider_aliases() {
        assert_eq!(resolve_model_alias("grok"), "grok-3");
        assert_eq!(resolve_model_alias("grok-mini"), "grok-3-mini");
        assert_eq!(resolve_model_alias("fast"), "openrouter/auto");
        assert_eq!(resolve_model_alias("hf"), "Qwen/Qwen3-Coder-Next:fastest");
        assert_eq!(
            resolve_model_alias("reasoner"),
            "deepseek-ai/DeepSeek-R1:fastest"
        );
        assert_eq!(resolve_model_alias("github"), "openai/gpt-4o-mini");
        assert_eq!(
            resolve_model_alias("deepseek-v3"),
            "deepseek/DeepSeek-V3-0324"
        );
    }

    #[test]
    fn detects_provider_from_model_name_first() {
        assert_eq!(detect_provider_kind("grok"), ProviderKind::Xai);
        assert_eq!(
            detect_provider_kind("openrouter/auto"),
            ProviderKind::OpenRouter
        );
        assert_eq!(
            detect_provider_kind("Qwen/Qwen3-Coder-Next:fastest"),
            ProviderKind::HuggingFace
        );
        assert_eq!(
            detect_provider_kind("openai/gpt-4.1-mini"),
            ProviderKind::GitHubModels
        );
        assert_eq!(
            detect_provider_kind("clues/coder-fast"),
            ProviderKind::OpenAi
        );
        assert_eq!(
            detect_provider_kind("deepseek-ai/DeepSeek-R1:fastest"),
            ProviderKind::HuggingFace
        );
    }

    #[test]
    fn keeps_existing_max_token_heuristic() {
        assert_eq!(max_tokens_for_model("openrouter/auto"), 64_000);
        assert_eq!(max_tokens_for_model("grok-3"), 64_000);
        assert_eq!(max_tokens_for_model("github-fast"), 2_048);
        assert_eq!(max_tokens_for_model("github"), 2_048);
        assert_eq!(max_tokens_for_model("deepseek-v3"), 1_024);
    }
}
