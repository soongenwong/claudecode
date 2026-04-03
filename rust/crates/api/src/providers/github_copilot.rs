use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::ApiError;
use crate::types::MessageRequest;
use runtime::{clear_credentials_entry, load_credentials_entry, save_credentials_entry};
use serde::{Deserialize, Serialize};

use super::openai_compat::{OpenAiCompatClient, OpenAiCompatConfig};

pub const DEFAULT_BASE_URL: &str = "https://api.individual.githubcopilot.com";
pub const GITHUB_TOKEN_ENV_VARS: &[&str] = &["COPILOT_GITHUB_TOKEN", "GH_TOKEN", "GITHUB_TOKEN"];

const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const COPILOT_TOKEN_URL: &str = "https://api.github.com/copilot_internal/v2/token";
const GITHUB_COPILOT_CLIENT_ID: &str = "Iv1.b507a08c87ecfe98";
const GITHUB_COPILOT_CREDENTIALS_KEY: &str = "githubCopilot";
const GITHUB_COPILOT_RUNTIME_KEY: &str = "githubCopilotRuntime";
const OPENCLAW_RUNTIME_CACHE_RELATIVE_PATH: &str =
    ".openclaw/credentials/github-copilot.token.json";
const OPENCLAW_AGENT_DIR_RELATIVE_PATH: &str = ".openclaw/agents";
const TOKEN_REFRESH_BUFFER_MS: u64 = 5 * 60 * 1000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GithubCopilotCredentials {
    pub github_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubCopilotRuntimeToken {
    pub token: String,
    pub expires_at: u64,
    #[serde(default)]
    pub updated_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedGithubCopilotAuth {
    pub api_key: String,
    pub base_url: String,
    pub expires_at: u64,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GithubDeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct GithubDeviceTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct GithubCopilotTokenResponse {
    token: String,
    expires_at: u64,
}

#[derive(Debug, Deserialize)]
struct OpenClawAuthProfiles {
    profiles: std::collections::BTreeMap<String, OpenClawAuthProfile>,
}

#[derive(Debug, Deserialize)]
struct OpenClawAuthProfile {
    provider: String,
    #[serde(rename = "type")]
    kind: String,
    token: Option<String>,
}

pub fn create_client(auth: ResolvedGithubCopilotAuth) -> OpenAiCompatClient {
    OpenAiCompatClient::new(auth.api_key, OpenAiCompatConfig::github_copilot())
        .with_base_url(auth.base_url)
}

pub fn client_from_env() -> Result<OpenAiCompatClient, ApiError> {
    let runtime = tokio::runtime::Runtime::new().map_err(ApiError::from)?;
    let auth = runtime.block_on(resolve_runtime_auth())?;
    Ok(create_client(auth))
}

pub fn save_github_token(github_token: &str) -> Result<(), ApiError> {
    save_credentials_entry(
        GITHUB_COPILOT_CREDENTIALS_KEY,
        &GithubCopilotCredentials {
            github_token: github_token.to_string(),
        },
    )
    .map_err(ApiError::from)
}

pub fn clear_github_token() -> Result<(), ApiError> {
    clear_credentials_entry(GITHUB_COPILOT_CREDENTIALS_KEY).map_err(ApiError::from)
}

pub fn load_saved_github_token() -> Result<Option<String>, ApiError> {
    if let Some(github_token) = read_first_non_empty_env(GITHUB_TOKEN_ENV_VARS)? {
        return Ok(Some(github_token));
    }

    if let Some(saved) =
        load_credentials_entry::<GithubCopilotCredentials>(GITHUB_COPILOT_CREDENTIALS_KEY)?
    {
        let github_token = saved.github_token.trim();
        if !github_token.is_empty() {
            return Ok(Some(github_token.to_string()));
        }
    }

    load_openclaw_github_token()
}

pub fn load_cached_runtime_token() -> Result<Option<ResolvedGithubCopilotAuth>, ApiError> {
    if let Some(cached) =
        load_credentials_entry::<GithubCopilotRuntimeToken>(GITHUB_COPILOT_RUNTIME_KEY)?
    {
        if is_runtime_token_usable(&cached, now_ms()) {
            let base_url = derive_base_url_from_token(&cached.token)
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
            return Ok(Some(ResolvedGithubCopilotAuth {
                api_key: cached.token,
                base_url,
                expires_at: cached.expires_at,
                source: "claw-cache".to_string(),
            }));
        }
    }

    if let Some(cached) = load_openclaw_runtime_token()? {
        if is_runtime_token_usable(&cached, now_ms()) {
            let base_url = derive_base_url_from_token(&cached.token)
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
            return Ok(Some(ResolvedGithubCopilotAuth {
                api_key: cached.token,
                base_url,
                expires_at: cached.expires_at,
                source: "openclaw-cache".to_string(),
            }));
        }
    }

    Ok(None)
}

pub async fn resolve_runtime_auth() -> Result<ResolvedGithubCopilotAuth, ApiError> {
    if let Some(cached) = load_cached_runtime_token()? {
        return Ok(cached);
    }

    let github_token = load_saved_github_token()?
        .ok_or_else(|| ApiError::missing_credentials("GitHub Copilot", GITHUB_TOKEN_ENV_VARS))?;

    refresh_runtime_auth(&github_token).await
}

pub async fn request_device_code() -> Result<GithubDeviceCodeResponse, ApiError> {
    let response = reqwest::Client::new()
        .post(DEVICE_CODE_URL)
        .header("accept", "application/json")
        .header("content-type", "application/x-www-form-urlencoded")
        .form(&[
            ("client_id", GITHUB_COPILOT_CLIENT_ID),
            ("scope", "read:user"),
        ])
        .send()
        .await
        .map_err(ApiError::from)?;
    let response = super::openai_compat::expect_success(response).await?;
    response
        .json::<GithubDeviceCodeResponse>()
        .await
        .map_err(ApiError::from)
}

pub async fn poll_for_access_token(device: &GithubDeviceCodeResponse) -> Result<String, ApiError> {
    let deadline = now_ms().saturating_add(device.expires_in.saturating_mul(1_000));
    let interval_ms = std::cmp::max(1_000, device.interval.saturating_mul(1_000));
    let client = reqwest::Client::new();

    while now_ms() < deadline {
        let response = client
            .post(ACCESS_TOKEN_URL)
            .header("accept", "application/json")
            .header("content-type", "application/x-www-form-urlencoded")
            .form(&[
                ("client_id", GITHUB_COPILOT_CLIENT_ID),
                ("device_code", device.device_code.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await
            .map_err(ApiError::from)?;
        let response = super::openai_compat::expect_success(response).await?;
        let payload = response
            .json::<GithubDeviceTokenResponse>()
            .await
            .map_err(ApiError::from)?;

        if let Some(access_token) = payload.access_token {
            let trimmed = access_token.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }

        match payload.error.as_deref() {
            Some("authorization_pending") => {
                tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            }
            Some("slow_down") => {
                tokio::time::sleep(Duration::from_millis(interval_ms + 2_000)).await;
            }
            Some("expired_token") => {
                return Err(ApiError::Auth(
                    "GitHub device code expired; run login-github-copilot again".to_string(),
                ));
            }
            Some("access_denied") => {
                return Err(ApiError::Auth("GitHub Copilot login cancelled".to_string()));
            }
            Some(other) => {
                return Err(ApiError::Auth(format!("GitHub device flow error: {other}")));
            }
            None => {
                tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            }
        }
    }

    Err(ApiError::Auth(
        "GitHub device code expired; run login-github-copilot again".to_string(),
    ))
}

pub async fn refresh_runtime_auth(
    github_token: &str,
) -> Result<ResolvedGithubCopilotAuth, ApiError> {
    let response = reqwest::Client::new()
        .get(COPILOT_TOKEN_URL)
        .header("accept", "application/json")
        .header("authorization", format!("Bearer {github_token}"))
        .header("user-agent", "GitHubCopilotChat/0.35.0")
        .header("editor-version", "vscode/1.107.0")
        .header("editor-plugin-version", "copilot-chat/0.35.0")
        .header("copilot-integration-id", "vscode-chat")
        .send()
        .await
        .map_err(ApiError::from)?;
    let response = super::openai_compat::expect_success(response).await?;
    let payload = response
        .json::<GithubCopilotTokenResponse>()
        .await
        .map_err(ApiError::from)?;

    let expires_at = normalize_expires_at(payload.expires_at)?;
    let base_url =
        derive_base_url_from_token(&payload.token).unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
    let cached = GithubCopilotRuntimeToken {
        token: payload.token.clone(),
        expires_at,
        updated_at: Some(now_ms()),
    };
    save_credentials_entry(GITHUB_COPILOT_RUNTIME_KEY, &cached).map_err(ApiError::from)?;

    Ok(ResolvedGithubCopilotAuth {
        api_key: payload.token,
        base_url,
        expires_at,
        source: "github".to_string(),
    })
}

pub fn derive_base_url_from_token(token: &str) -> Option<String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }
    let proxy = trimmed
        .split(';')
        .find_map(|segment| segment.trim().strip_prefix("proxy-ep="))?;
    let host = proxy
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .replace("proxy.", "api.");
    (!host.is_empty()).then(|| format!("https://{host}"))
}

pub fn dynamic_headers_for_request(request: &MessageRequest) -> Vec<(&'static str, String)> {
    let initiator = request.messages.last().map_or("user", |message| {
        if message.role == "user" {
            "user"
        } else {
            "agent"
        }
    });
    vec![
        ("X-Initiator", initiator.to_string()),
        ("Openai-Intent", "conversation-edits".to_string()),
    ]
}

fn load_openclaw_runtime_token() -> Result<Option<GithubCopilotRuntimeToken>, ApiError> {
    let Some(home) = home_dir() else {
        return Ok(None);
    };
    let path = home.join(OPENCLAW_RUNTIME_CACHE_RELATIVE_PATH);
    read_json_file::<GithubCopilotRuntimeToken>(&path).map_err(ApiError::from)
}

fn load_openclaw_github_token() -> Result<Option<String>, ApiError> {
    let Some(home) = home_dir() else {
        return Ok(None);
    };
    let agents_dir = home.join(OPENCLAW_AGENT_DIR_RELATIVE_PATH);
    let entries = match fs::read_dir(&agents_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(ApiError::from(error)),
    };

    for entry in entries {
        let entry = entry.map_err(ApiError::from)?;
        let path = entry.path().join("agent").join("auth-profiles.json");
        let Some(root) = read_json_file::<OpenClawAuthProfiles>(&path).map_err(ApiError::from)?
        else {
            continue;
        };
        for profile in root.profiles.into_values() {
            if profile.provider == "github-copilot" && profile.kind == "token" {
                if let Some(token) = profile.token {
                    let trimmed = token.trim();
                    if !trimmed.is_empty() {
                        return Ok(Some(trimmed.to_string()));
                    }
                }
            }
        }
    }

    Ok(None)
}

fn read_json_file<T>(path: &Path) -> std::io::Result<Option<T>>
where
    T: for<'de> Deserialize<'de>,
{
    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map(Some)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn read_first_non_empty_env(keys: &[&str]) -> Result<Option<String>, ApiError> {
    for key in keys {
        match std::env::var(key) {
            Ok(value) if !value.trim().is_empty() => return Ok(Some(value)),
            Ok(_) | Err(std::env::VarError::NotPresent) => {}
            Err(error) => return Err(ApiError::from(error)),
        }
    }
    Ok(None)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn is_runtime_token_usable(cached: &GithubCopilotRuntimeToken, now: u64) -> bool {
    cached.expires_at.saturating_sub(now) > TOKEN_REFRESH_BUFFER_MS
}

fn normalize_expires_at(value: u64) -> Result<u64, ApiError> {
    if value == 0 {
        return Err(ApiError::Auth(
            "GitHub Copilot token response missing expires_at".to_string(),
        ));
    }
    Ok(if value < 100_000_000_000 {
        value.saturating_mul(1_000)
    } else {
        value
    })
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::{
        derive_base_url_from_token, dynamic_headers_for_request, is_runtime_token_usable,
        normalize_expires_at, GithubCopilotRuntimeToken,
    };
    use crate::types::{InputMessage, MessageRequest};

    #[test]
    fn derives_base_url_from_proxy_endpoint() {
        let token = "tid=1;exp=9999999999;proxy-ep=proxy.individual.githubcopilot.com;foo=bar";
        assert_eq!(
            derive_base_url_from_token(token).as_deref(),
            Some("https://api.individual.githubcopilot.com")
        );
    }

    #[test]
    fn normalizes_expires_at_seconds_and_millis() {
        assert_eq!(
            normalize_expires_at(1_775_176_140).expect("seconds should normalize"),
            1_775_176_140_000
        );
        assert_eq!(
            normalize_expires_at(1_775_176_140_000).expect("millis should stay millis"),
            1_775_176_140_000
        );
    }

    #[test]
    fn runtime_token_usability_requires_buffer() {
        let cached = GithubCopilotRuntimeToken {
            token: "abc".to_string(),
            expires_at: 1_000_000,
            updated_at: Some(1),
        };
        assert!(is_runtime_token_usable(&cached, 600_000));
        assert!(!is_runtime_token_usable(&cached, 800_000));
    }

    #[test]
    fn dynamic_headers_switch_to_agent_after_assistant_turn() {
        let request = MessageRequest {
            model: "github-copilot/gpt-4o".to_string(),
            max_tokens: 128,
            messages: vec![
                InputMessage::user_text("hi"),
                InputMessage {
                    role: "assistant".to_string(),
                    content: vec![],
                },
            ],
            system: None,
            tools: None,
            tool_choice: None,
            stream: false,
        };

        let headers = dynamic_headers_for_request(&request);
        assert!(headers
            .iter()
            .any(|(key, value)| *key == "X-Initiator" && value == "agent"));
        assert!(headers
            .iter()
            .any(|(key, value)| *key == "Openai-Intent" && value == "conversation-edits"));
    }
}
