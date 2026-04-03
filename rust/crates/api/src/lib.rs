mod client;
mod error;
mod providers;
mod sse;
mod types;

pub use client::{
    oauth_token_is_expired, read_base_url, read_xai_base_url, resolve_saved_oauth_token,
    resolve_startup_auth_source, MessageStream, OAuthTokenSet, ProviderClient,
};
pub use error::ApiError;
pub use providers::claw_provider::{AuthSource, ClawApiClient, ClawApiClient as ApiClient};
pub use providers::github_copilot::{
    cached_model_availability_is_fresh, load_cached_model_availability,
    clear_github_token as clear_github_copilot_token, load_saved_github_token,
    poll_for_access_token as poll_for_github_copilot_access_token,
    request_device_code as request_github_copilot_device_code,
    refresh_model_availability as refresh_github_copilot_model_availability,
    resolve_model_availability as resolve_github_copilot_model_availability,
    resolve_runtime_auth as resolve_github_copilot_runtime_auth,
    save_github_token as save_github_copilot_token, GithubCopilotModelAvailability,
    GithubDeviceCodeResponse,
};
pub use providers::openai_compat::{OpenAiCompatClient, OpenAiCompatConfig};
pub use providers::{
    detect_provider_kind, max_tokens_for_model, resolve_model_alias, ProviderKind,
};
pub use sse::{parse_frame, SseParser};
pub use types::{
    ContentBlockDelta, ContentBlockDeltaEvent, ContentBlockStartEvent, ContentBlockStopEvent,
    InputContentBlock, InputMessage, MessageDelta, MessageDeltaEvent, MessageRequest,
    MessageResponse, MessageStartEvent, MessageStopEvent, OutputContentBlock, StreamEvent,
    ToolChoice, ToolDefinition, ToolResultContentBlock, Usage,
};
