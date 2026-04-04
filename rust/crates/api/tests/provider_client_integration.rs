use std::ffi::OsString;
use std::sync::{Mutex, OnceLock};

use api::{
    read_github_models_base_url, read_hugging_face_base_url, read_openrouter_base_url,
    read_xai_base_url, ApiError, ProviderClient, ProviderKind,
};

#[test]
fn provider_client_routes_grok_aliases_through_xai() {
    let _lock = env_lock();
    let _xai_api_key = EnvVarGuard::set("XAI_API_KEY", Some("xai-test-key"));

    let client = ProviderClient::from_model("grok-mini").expect("grok alias should resolve");

    assert_eq!(client.provider_kind(), ProviderKind::Xai);
}

#[test]
fn provider_client_routes_openrouter_aliases() {
    let _lock = env_lock();
    let _openrouter_api_key = EnvVarGuard::set("OPENROUTER_API_KEY", Some("openrouter-test-key"));

    let client =
        ProviderClient::from_model("openrouter").expect("openrouter alias should resolve");

    assert_eq!(client.provider_kind(), ProviderKind::OpenRouter);
}

#[test]
fn provider_client_routes_hugging_face_aliases() {
    let _lock = env_lock();
    let _hf_token = EnvVarGuard::set("HF_TOKEN", Some("hf-test-key"));

    let client = ProviderClient::from_model("hf").expect("hf alias should resolve");

    assert_eq!(client.provider_kind(), ProviderKind::HuggingFace);
}

#[test]
fn provider_client_routes_github_models_aliases() {
    let _lock = env_lock();
    let _github_token = EnvVarGuard::set("GITHUB_TOKEN", Some("ghm-test-key"));

    let client =
        ProviderClient::from_model("deepseek-v3").expect("github models alias should resolve");

    assert_eq!(client.provider_kind(), ProviderKind::GitHubModels);
}

#[test]
fn provider_client_routes_gateway_profiles_through_openai_compat() {
    let _lock = env_lock();
    let _openai_api_key = EnvVarGuard::set("OPENAI_API_KEY", Some("gateway-test-key"));

    let client =
        ProviderClient::from_model("clues/coder-fast").expect("gateway profile should resolve");
    assert_eq!(client.provider_kind(), ProviderKind::OpenAi);
}

#[test]
fn provider_client_reports_missing_xai_credentials_for_grok_models() {
    let _lock = env_lock();
    let _xai_api_key = EnvVarGuard::set("XAI_API_KEY", None);

    let error = ProviderClient::from_model("grok-3")
        .expect_err("grok requests without XAI_API_KEY should fail fast");

    match error {
        ApiError::MissingCredentials { provider, env_vars } => {
            assert_eq!(provider, "xAI");
            assert_eq!(env_vars, &["XAI_API_KEY"]);
        }
        other => panic!("expected missing xAI credentials, got {other:?}"),
    }
}

#[test]
fn provider_client_reports_missing_openrouter_credentials() {
    let _lock = env_lock();
    let _openrouter_api_key = EnvVarGuard::set("OPENROUTER_API_KEY", None);
    let _openai_api_key = EnvVarGuard::set("OPENAI_API_KEY", None);

    let error = ProviderClient::from_model("openrouter/auto")
        .expect_err("openrouter requests without credentials should fail fast");

    match error {
        ApiError::MissingCredentials { provider, env_vars } => {
            assert_eq!(provider, "OpenRouter");
            assert_eq!(env_vars, &["OPENROUTER_API_KEY", "OPENAI_API_KEY"]);
        }
        other => panic!("expected missing OpenRouter credentials, got {other:?}"),
    }
}

#[test]
fn read_xai_base_url_prefers_env_override() {
    let _lock = env_lock();
    let _xai_base_url = EnvVarGuard::set("XAI_BASE_URL", Some("https://example.xai.test/v1"));

    assert_eq!(read_xai_base_url(), "https://example.xai.test/v1");
}

#[test]
fn read_openrouter_base_url_prefers_provider_specific_override() {
    let _lock = env_lock();
    let _openrouter_base_url =
        EnvVarGuard::set("OPENROUTER_BASE_URL", Some("https://openrouter.example.test/v1"));
    let _openai_base_url = EnvVarGuard::set("OPENAI_BASE_URL", Some("https://fallback.test/v1"));

    assert_eq!(
        read_openrouter_base_url(),
        "https://openrouter.example.test/v1"
    );
}

#[test]
fn read_hugging_face_base_url_uses_fallback_aliases() {
    let _lock = env_lock();
    let _hf_base_url = EnvVarGuard::set("HF_BASE_URL", Some("https://hf.example.test/v1"));
    let _huggingface_base_url = EnvVarGuard::set("HUGGINGFACE_BASE_URL", None);
    let _openai_base_url = EnvVarGuard::set("OPENAI_BASE_URL", None);

    assert_eq!(read_hugging_face_base_url(), "https://hf.example.test/v1");
}

#[test]
fn read_github_models_base_url_prefers_env_override() {
    let _lock = env_lock();
    let _github_models_base_url = EnvVarGuard::set(
        "GITHUB_MODELS_BASE_URL",
        Some("https://github-models.example.test/inference"),
    );

    assert_eq!(
        read_github_models_base_url(),
        "https://github-models.example.test/inference"
    );
}

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: Option<&str>) -> Self {
        let original = std::env::var_os(key);
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}
