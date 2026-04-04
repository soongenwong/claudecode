mod client;
mod error;
mod oauth;
mod providers;
mod sse;
mod types;

pub use client::{
    read_github_models_base_url, read_hugging_face_base_url, read_openrouter_base_url,
    read_xai_base_url, MessageStream, ProviderClient,
};
pub use error::ApiError;
pub use oauth::{oauth_token_is_expired, resolve_saved_oauth_token, OAuthHttpClient};
pub use providers::openai_compat::{OpenAiCompatClient, OpenAiCompatConfig};
pub use providers::{
    detect_provider_kind, max_tokens_for_model, resolve_model_alias, ProviderKind,
};
pub use sse::{parse_frame, SseParser};
pub use runtime::OAuthTokenSet;
pub use types::{
    ContentBlockDelta, ContentBlockDeltaEvent, ContentBlockStartEvent, ContentBlockStopEvent,
    InputContentBlock, InputMessage, MessageDelta, MessageDeltaEvent, MessageRequest,
    MessageResponse, MessageStartEvent, MessageStopEvent, OutputContentBlock, StreamEvent,
    ToolChoice, ToolDefinition, ToolResultContentBlock, Usage,
};
