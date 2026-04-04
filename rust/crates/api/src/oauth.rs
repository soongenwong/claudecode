use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use runtime::{
    load_oauth_credentials, save_oauth_credentials, OAuthConfig, OAuthRefreshRequest,
    OAuthTokenExchangeRequest, OAuthTokenSet,
};
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Debug, Clone)]
pub struct OAuthHttpClient {
    http: reqwest::Client,
}

impl OAuthHttpClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    pub async fn exchange_oauth_code(
        &self,
        config: &OAuthConfig,
        request: &OAuthTokenExchangeRequest,
    ) -> Result<OAuthTokenSet, ApiError> {
        self.post_token_form(&config.token_url, &request.form_params())
            .await
    }

    pub async fn refresh_oauth_token(
        &self,
        config: &OAuthConfig,
        request: &OAuthRefreshRequest,
    ) -> Result<OAuthTokenSet, ApiError> {
        self.post_token_form(&config.token_url, &request.form_params())
            .await
    }

    async fn post_token_form(
        &self,
        token_url: &str,
        form_params: &BTreeMap<&str, String>,
    ) -> Result<OAuthTokenSet, ApiError> {
        let response = self
            .http
            .post(token_url)
            .header("content-type", "application/x-www-form-urlencoded")
            .form(form_params)
            .send()
            .await
            .map_err(ApiError::from)?;
        let response = expect_success(response).await?;
        response.json::<OAuthTokenSet>().await.map_err(ApiError::from)
    }
}

impl Default for OAuthHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn oauth_token_is_expired(token_set: &OAuthTokenSet) -> bool {
    token_set
        .expires_at
        .is_some_and(|expires_at| expires_at <= now_unix_timestamp())
}

pub fn resolve_saved_oauth_token(config: &OAuthConfig) -> Result<Option<OAuthTokenSet>, ApiError> {
    let Some(token_set) = load_oauth_credentials().map_err(ApiError::from)? else {
        return Ok(None);
    };
    resolve_saved_oauth_token_set(config, token_set).map(Some)
}

fn resolve_saved_oauth_token_set(
    config: &OAuthConfig,
    token_set: OAuthTokenSet,
) -> Result<OAuthTokenSet, ApiError> {
    if !oauth_token_is_expired(&token_set) {
        return Ok(token_set);
    }
    let Some(refresh_token) = token_set.refresh_token.clone() else {
        return Err(ApiError::ExpiredOAuthToken);
    };
    let client = OAuthHttpClient::new();
    let refreshed = client_runtime_block_on(async {
        client
            .refresh_oauth_token(
                config,
                &OAuthRefreshRequest::from_config(
                    config,
                    refresh_token,
                    Some(token_set.scopes.clone()),
                ),
            )
            .await
    })?;
    let resolved = OAuthTokenSet {
        access_token: refreshed.access_token,
        refresh_token: refreshed.refresh_token.or(token_set.refresh_token),
        expires_at: refreshed.expires_at,
        scopes: refreshed.scopes,
    };
    save_oauth_credentials(&resolved).map_err(ApiError::from)?;
    Ok(resolved)
}

async fn expect_success(response: reqwest::Response) -> Result<reqwest::Response, ApiError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let body = response.text().await.unwrap_or_default();
    let parsed_error = serde_json::from_str::<ErrorEnvelope>(&body).ok();

    Err(ApiError::Api {
        status,
        error_type: parsed_error
            .as_ref()
            .and_then(|error| error.error.error_type.clone()),
        message: parsed_error
            .as_ref()
            .and_then(|error| error.error.message.clone()),
        body,
        retryable: false,
    })
}

fn client_runtime_block_on<F>(future: F) -> Result<OAuthTokenSet, ApiError>
where
    F: std::future::Future<Output = Result<OAuthTokenSet, ApiError>>,
{
    tokio::runtime::Runtime::new()
        .map_err(ApiError::from)?
        .block_on(future)
}

#[must_use]
fn now_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("current time should be after unix epoch")
        .as_secs()
}

#[derive(Debug, Clone, Deserialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Clone, Deserialize)]
struct ErrorBody {
    #[serde(default, rename = "type")]
    error_type: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::time::{SystemTime, UNIX_EPOCH};

    use runtime::{clear_oauth_credentials, save_oauth_credentials, OAuthConfig, OAuthTokenSet};
    use super::{oauth_token_is_expired, resolve_saved_oauth_token};

    #[test]
    fn oauth_token_expiry_uses_expires_at_timestamp() {
        assert!(oauth_token_is_expired(&OAuthTokenSet {
            access_token: "expired".to_string(),
            refresh_token: None,
            expires_at: Some(1),
            scopes: Vec::new(),
        }));
        assert!(!oauth_token_is_expired(&OAuthTokenSet {
            access_token: "fresh".to_string(),
            refresh_token: None,
            expires_at: Some(now_secs() + 60),
            scopes: Vec::new(),
        }));
    }

    #[test]
    fn resolve_saved_oauth_token_refreshes_expired_credentials() {
        let config_home = temp_config_home();
        std::env::set_var("CLUES_CONFIG_HOME", &config_home);
        save_oauth_credentials(&OAuthTokenSet {
            access_token: "expired-token".to_string(),
            refresh_token: Some("refresh-token".to_string()),
            expires_at: Some(1),
            scopes: vec!["scope:a".to_string()],
        })
        .expect("save expired oauth credentials");

        let token_url = spawn_token_server(
            "{\"access_token\":\"refreshed-token\",\"expires_at\":9999999999,\"scopes\":[\"scope:a\"]}",
        );
        let resolved = resolve_saved_oauth_token(&sample_oauth_config(token_url))
            .expect("resolve saved token")
            .expect("token set should exist");
        assert_eq!(resolved.access_token, "refreshed-token");
        assert_eq!(resolved.refresh_token.as_deref(), Some("refresh-token"));

        clear_oauth_credentials().expect("clear credentials");
        std::env::remove_var("CLUES_CONFIG_HOME");
        std::fs::remove_dir_all(config_home).expect("cleanup temp config home");
    }

    fn sample_oauth_config(token_url: String) -> OAuthConfig {
        OAuthConfig {
            client_id: "client-id".to_string(),
            authorize_url: "https://example.test/oauth/authorize".to_string(),
            token_url,
            callback_port: None,
            manual_redirect_url: None,
            scopes: vec!["scope:a".to_string()],
        }
    }

    fn temp_config_home() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("api-oauth-{nanos}"));
        std::fs::create_dir_all(&path).expect("create temp config home");
        path
    }

    fn spawn_token_server(body: &'static str) -> String {
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let address = listener.local_addr().expect("listener addr");
        std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().expect("accept");
            let mut buffer = [0_u8; 4096];
            let bytes_read = socket.read(&mut buffer).expect("read request");
            let request = String::from_utf8_lossy(&buffer[..bytes_read]).into_owned();
            let captured = parse_request(&request);
            assert_eq!(
                captured
                    .headers
                    .get("content-type")
                    .map(String::as_str),
                Some("application/x-www-form-urlencoded")
            );
            assert!(captured.body.contains("grant_type=refresh_token"));
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            socket
                .write_all(response.as_bytes())
                .expect("write response");
        });
        format!("http://{address}/token")
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CapturedRequest {
        headers: HashMap<String, String>,
        body: String,
    }

    fn parse_request(raw_request: &str) -> CapturedRequest {
        let mut lines = raw_request.lines();
        let _ = lines.next();
        let mut headers = HashMap::new();
        let mut content_length = 0_usize;

        for line in lines.by_ref() {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_ascii_lowercase();
                let value = value.trim().to_string();
                if key == "content-length" {
                    content_length = value.parse::<usize>().unwrap_or(0);
                }
                headers.insert(key, value);
            }
        }

        let body = if content_length == 0 {
            String::new()
        } else {
            raw_request
                .split("\r\n\r\n")
                .nth(1)
                .unwrap_or_default()
                .chars()
                .take(content_length)
                .collect()
        };

        CapturedRequest { headers, body }
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after unix epoch")
            .as_secs()
    }
}
