use crate::error::ApiError;
use crate::providers::openai_compat::{self, OpenAiCompatClient, OpenAiCompatConfig};
use crate::providers::{self, Provider, ProviderKind};
use crate::types::{MessageRequest, MessageResponse, StreamEvent};

async fn send_via_provider<P: Provider>(
    provider: &P,
    request: &MessageRequest,
) -> Result<MessageResponse, ApiError> {
    provider.send_message(request).await
}

async fn stream_via_provider<P: Provider>(
    provider: &P,
    request: &MessageRequest,
) -> Result<P::Stream, ApiError> {
    provider.stream_message(request).await
}

#[derive(Debug, Clone)]
pub enum ProviderClient {
    Xai(OpenAiCompatClient),
    OpenAi(OpenAiCompatClient),
    OpenRouter(OpenAiCompatClient),
    HuggingFace(OpenAiCompatClient),
    GitHubModels(OpenAiCompatClient),
}

impl ProviderClient {
    pub fn from_model(model: &str) -> Result<Self, ApiError> {
        let resolved_model = providers::resolve_model_alias(model);
        match providers::detect_provider_kind(&resolved_model) {
            ProviderKind::Xai => Ok(Self::Xai(OpenAiCompatClient::from_env(
                OpenAiCompatConfig::xai(),
            )?)),
            ProviderKind::OpenAi => Ok(Self::OpenAi(OpenAiCompatClient::from_env(
                OpenAiCompatConfig::openai(),
            )?)),
            ProviderKind::OpenRouter => Ok(Self::OpenRouter(OpenAiCompatClient::from_env(
                OpenAiCompatConfig::openrouter(),
            )?)),
            ProviderKind::HuggingFace => Ok(Self::HuggingFace(OpenAiCompatClient::from_env(
                OpenAiCompatConfig::hugging_face(),
            )?)),
            ProviderKind::GitHubModels => Ok(Self::GitHubModels(
                OpenAiCompatClient::from_env(OpenAiCompatConfig::github_models())?,
            )),
        }
    }

    #[must_use]
    pub const fn provider_kind(&self) -> ProviderKind {
        match self {
            Self::Xai(_) => ProviderKind::Xai,
            Self::OpenAi(_) => ProviderKind::OpenAi,
            Self::OpenRouter(_) => ProviderKind::OpenRouter,
            Self::HuggingFace(_) => ProviderKind::HuggingFace,
            Self::GitHubModels(_) => ProviderKind::GitHubModels,
        }
    }

    pub async fn send_message(
        &self,
        request: &MessageRequest,
    ) -> Result<MessageResponse, ApiError> {
        match self {
            Self::Xai(client)
            | Self::OpenAi(client)
            | Self::OpenRouter(client)
            | Self::HuggingFace(client)
            | Self::GitHubModels(client) => send_via_provider(client, request).await,
        }
    }

    pub async fn stream_message(
        &self,
        request: &MessageRequest,
    ) -> Result<MessageStream, ApiError> {
        match self {
            Self::Xai(client)
            | Self::OpenAi(client)
            | Self::OpenRouter(client)
            | Self::HuggingFace(client)
            | Self::GitHubModels(client) => stream_via_provider(client, request)
                .await
                .map(MessageStream::OpenAiCompat),
        }
    }
}

#[derive(Debug)]
pub enum MessageStream {
    OpenAiCompat(openai_compat::MessageStream),
}

impl MessageStream {
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::OpenAiCompat(stream) => stream.request_id(),
        }
    }

    pub async fn next_event(&mut self) -> Result<Option<StreamEvent>, ApiError> {
        match self {
            Self::OpenAiCompat(stream) => stream.next_event().await,
        }
    }
}

#[must_use]
pub fn read_xai_base_url() -> String {
    openai_compat::read_base_url(OpenAiCompatConfig::xai())
}

#[must_use]
pub fn read_openrouter_base_url() -> String {
    openai_compat::read_base_url(OpenAiCompatConfig::openrouter())
}

#[must_use]
pub fn read_hugging_face_base_url() -> String {
    openai_compat::read_base_url(OpenAiCompatConfig::hugging_face())
}

#[must_use]
pub fn read_github_models_base_url() -> String {
    openai_compat::read_base_url(OpenAiCompatConfig::github_models())
}

#[cfg(test)]
mod tests {
    use crate::providers::{detect_provider_kind, resolve_model_alias, ProviderKind};

    #[test]
    fn resolves_existing_and_provider_aliases() {
        assert_eq!(resolve_model_alias("grok"), "grok-3");
        assert_eq!(resolve_model_alias("grok-mini"), "grok-3-mini");
        assert_eq!(resolve_model_alias("fast"), "openrouter/auto");
        assert_eq!(resolve_model_alias("hf"), "Qwen/Qwen3-Coder-Next:fastest");
        assert_eq!(resolve_model_alias("reasoner"), "deepseek-ai/DeepSeek-R1:fastest");
        assert_eq!(resolve_model_alias("github"), "openai/gpt-4o-mini");
    }

    #[test]
    fn provider_detection_prefers_model_family() {
        assert_eq!(detect_provider_kind("grok-3"), ProviderKind::Xai);
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
        assert_eq!(detect_provider_kind("clues/coder-fast"), ProviderKind::OpenAi);
    }
}
