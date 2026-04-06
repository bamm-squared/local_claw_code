#![allow(clippy::cast_possible_truncation)]
use std::future::Future;
use std::pin::Pin;

use runtime::{active_provider_override, RuntimeProviderConfig};
use serde::Serialize;

use crate::error::ApiError;
use crate::types::{MessageRequest, MessageResponse};

pub mod anthropic;
pub mod openai_compat;

#[allow(dead_code)]
pub type ProviderFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, ApiError>> + Send + 'a>>;

#[allow(dead_code)]
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
    Anthropic,
    OpenAi,
    Xai,
    Gemini,
    OpenAiCompatible,
    AnthropicCompatible,
    Ollama,
}

impl ProviderKind {
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Anthropic => "Anthropic",
            Self::OpenAi => "OpenAI",
            Self::Xai => "xAI",
            Self::Gemini => "Gemini",
            Self::OpenAiCompatible => "OpenAI-compatible",
            Self::AnthropicCompatible => "Anthropic-compatible",
            Self::Ollama => "Ollama",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderTransport {
    Anthropic,
    OpenAiCompat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderMetadata {
    pub id: &'static str,
    pub display_name: &'static str,
    pub provider: ProviderKind,
    pub transport: ProviderTransport,
    pub auth_env: &'static str,
    pub auth_env_secondary: Option<&'static str>,
    pub base_url_env: &'static str,
    pub default_base_url: Option<&'static str>,
    pub supports_saved_oauth: bool,
    pub fallback_auth: Option<anthropic::StaticAuthFallback>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelTokenLimit {
    pub max_output_tokens: u32,
    pub context_window_tokens: u32,
}

pub const SUPPORTED_PROVIDER_IDS: &[&str] = &[
    "anthropic",
    "openai",
    "xai",
    "gemini",
    "openai-compatible",
    "anthropic-compatible",
    "ollama",
];

const fn anthropic_metadata() -> ProviderMetadata {
    ProviderMetadata {
        id: "anthropic",
        display_name: "Anthropic",
        provider: ProviderKind::Anthropic,
        transport: ProviderTransport::Anthropic,
        auth_env: "ANTHROPIC_AUTH_TOKEN",
        auth_env_secondary: Some("ANTHROPIC_API_KEY"),
        base_url_env: "ANTHROPIC_BASE_URL",
        default_base_url: Some(anthropic::DEFAULT_BASE_URL),
        supports_saved_oauth: true,
        fallback_auth: None,
    }
}

const fn openai_metadata() -> ProviderMetadata {
    ProviderMetadata {
        id: "openai",
        display_name: "OpenAI",
        provider: ProviderKind::OpenAi,
        transport: ProviderTransport::OpenAiCompat,
        auth_env: "OPENAI_API_KEY",
        auth_env_secondary: None,
        base_url_env: "OPENAI_BASE_URL",
        default_base_url: Some(openai_compat::DEFAULT_OPENAI_BASE_URL),
        supports_saved_oauth: false,
        fallback_auth: None,
    }
}

const fn xai_metadata() -> ProviderMetadata {
    ProviderMetadata {
        id: "xai",
        display_name: "xAI",
        provider: ProviderKind::Xai,
        transport: ProviderTransport::OpenAiCompat,
        auth_env: "XAI_API_KEY",
        auth_env_secondary: None,
        base_url_env: "XAI_BASE_URL",
        default_base_url: Some(openai_compat::DEFAULT_XAI_BASE_URL),
        supports_saved_oauth: false,
        fallback_auth: None,
    }
}

const fn gemini_metadata() -> ProviderMetadata {
    ProviderMetadata {
        id: "gemini",
        display_name: "Gemini",
        provider: ProviderKind::Gemini,
        transport: ProviderTransport::OpenAiCompat,
        auth_env: "GEMINI_API_KEY",
        auth_env_secondary: None,
        base_url_env: "GEMINI_BASE_URL",
        default_base_url: None,
        supports_saved_oauth: false,
        fallback_auth: None,
    }
}

const fn openai_compatible_metadata() -> ProviderMetadata {
    ProviderMetadata {
        id: "openai-compatible",
        display_name: "OpenAI-compatible",
        provider: ProviderKind::OpenAiCompatible,
        transport: ProviderTransport::OpenAiCompat,
        auth_env: "OPENAI_COMPAT_API_KEY",
        auth_env_secondary: None,
        base_url_env: "OPENAI_COMPAT_BASE_URL",
        default_base_url: None,
        supports_saved_oauth: false,
        fallback_auth: None,
    }
}

const fn anthropic_compatible_metadata() -> ProviderMetadata {
    ProviderMetadata {
        id: "anthropic-compatible",
        display_name: "Anthropic-compatible",
        provider: ProviderKind::AnthropicCompatible,
        transport: ProviderTransport::Anthropic,
        auth_env: "ANTHROPIC_COMPAT_AUTH_TOKEN",
        auth_env_secondary: Some("ANTHROPIC_COMPAT_API_KEY"),
        base_url_env: "ANTHROPIC_COMPAT_BASE_URL",
        default_base_url: None,
        supports_saved_oauth: false,
        fallback_auth: None,
    }
}

const fn ollama_metadata() -> ProviderMetadata {
    ProviderMetadata {
        id: "ollama",
        display_name: "Ollama",
        provider: ProviderKind::Ollama,
        transport: ProviderTransport::Anthropic,
        auth_env: "OLLAMA_AUTH_TOKEN",
        auth_env_secondary: Some("OLLAMA_API_KEY"),
        base_url_env: "OLLAMA_BASE_URL",
        default_base_url: Some("http://localhost:11434"),
        supports_saved_oauth: false,
        fallback_auth: Some(anthropic::StaticAuthFallback::BearerToken("ollama")),
    }
}

fn effective_provider_config(
    provider_config: Option<&RuntimeProviderConfig>,
) -> RuntimeProviderConfig {
    active_provider_override()
        .unwrap_or_default()
        .merged_with(&provider_config.cloned().unwrap_or_default())
}

#[must_use]
pub fn resolve_model_alias(model: &str) -> String {
    let trimmed = model.trim();
    match trimmed.to_ascii_lowercase().as_str() {
        "opus" => "claude-opus-4-6".to_string(),
        "sonnet" => "claude-sonnet-4-6".to_string(),
        "haiku" => "claude-haiku-4-5-20251213".to_string(),
        "grok" | "grok-3" => "grok-3".to_string(),
        "grok-mini" | "grok-3-mini" => "grok-3-mini".to_string(),
        "grok-2" => "grok-2".to_string(),
        _ => trimmed.to_string(),
    }
}

#[must_use]
pub fn provider_metadata_by_id(provider_id: &str) -> Option<ProviderMetadata> {
    match provider_id.trim().to_ascii_lowercase().as_str() {
        "anthropic" | "claude" => Some(anthropic_metadata()),
        "openai" | "chatgpt" => Some(openai_metadata()),
        "xai" | "grok" => Some(xai_metadata()),
        "gemini" | "google" => Some(gemini_metadata()),
        "openai-compatible" | "openai_compat" | "openai-compat" => {
            Some(openai_compatible_metadata())
        }
        "anthropic-compatible" | "anthropic_compat" | "anthropic-compat" => {
            Some(anthropic_compatible_metadata())
        }
        "ollama" => Some(ollama_metadata()),
        _ => None,
    }
}

#[must_use]
pub fn metadata_for_model(model: &str) -> Option<ProviderMetadata> {
    let canonical = resolve_model_alias(model).to_ascii_lowercase();
    if canonical.starts_with("claude") {
        return Some(anthropic_metadata());
    }
    if canonical.starts_with("grok") {
        return Some(xai_metadata());
    }
    if canonical.starts_with("gemini") {
        return Some(gemini_metadata());
    }
    if canonical.starts_with("gpt")
        || canonical.starts_with("chatgpt")
        || canonical.starts_with("o1")
        || canonical.starts_with("o3")
        || canonical.starts_with("o4")
    {
        return Some(openai_metadata());
    }
    None
}

pub fn resolve_provider_metadata(
    model: &str,
    provider_config: Option<&RuntimeProviderConfig>,
) -> Result<ProviderMetadata, ApiError> {
    let effective = effective_provider_config(provider_config);
    if let Some(provider_id) = effective.id() {
        return provider_metadata_by_id(provider_id).ok_or(ApiError::UnsupportedProvider {
            provider: provider_id.to_string(),
            supported: SUPPORTED_PROVIDER_IDS,
        });
    }

    if let Some(metadata) = metadata_for_model(model) {
        return Ok(metadata);
    }

    let configured = configured_provider_metadata()?;
    match configured.as_slice() {
        [metadata] => Ok(*metadata),
        [] => Err(ApiError::ProviderSelectionRequired {
            model: resolve_model_alias(model),
            supported: SUPPORTED_PROVIDER_IDS,
        }),
        many => Err(ApiError::AmbiguousProviderSelection {
            model: resolve_model_alias(model),
            configured: many.iter().map(|metadata| metadata.id).collect(),
        }),
    }
}

pub fn base_url_for_provider(
    metadata: ProviderMetadata,
    provider_config: Option<&RuntimeProviderConfig>,
) -> Result<String, ApiError> {
    let effective = effective_provider_config(provider_config);
    if let Some(base_url) = effective.base_url() {
        return Ok(base_url.to_string());
    }
    match metadata.transport {
        ProviderTransport::Anthropic => {
            anthropic::read_base_url_with_config(anthropic_config_for_metadata(metadata))
        }
        ProviderTransport::OpenAiCompat => {
            openai_compat::read_base_url(openai_compat_config_for_metadata(metadata))
        }
    }
}

#[must_use]
pub fn anthropic_config_for_metadata(metadata: ProviderMetadata) -> anthropic::AnthropicEnvConfig {
    anthropic::AnthropicEnvConfig {
        provider_name: metadata.display_name,
        api_key_env: metadata.auth_env_secondary.unwrap_or(metadata.auth_env),
        auth_token_env: metadata.auth_env,
        credential_env_vars: match metadata.id {
            "anthropic" => &["ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY"],
            "anthropic-compatible" => &["ANTHROPIC_COMPAT_AUTH_TOKEN", "ANTHROPIC_COMPAT_API_KEY"],
            "ollama" => &["OLLAMA_AUTH_TOKEN", "OLLAMA_API_KEY"],
            other => panic!("unsupported anthropic provider metadata: {other}"),
        },
        base_url_env: metadata.base_url_env,
        default_base_url: metadata.default_base_url,
        allow_saved_oauth: metadata.supports_saved_oauth,
        fallback_auth: metadata.fallback_auth,
    }
}

#[must_use]
pub fn openai_compat_config_for_metadata(
    metadata: ProviderMetadata,
) -> openai_compat::OpenAiCompatConfig {
    match metadata.id {
        "openai" => openai_compat::OpenAiCompatConfig::openai(),
        "xai" => openai_compat::OpenAiCompatConfig::xai(),
        "gemini" => openai_compat::OpenAiCompatConfig::gemini(),
        "openai-compatible" => openai_compat::OpenAiCompatConfig::openai_compatible(),
        other => panic!("unsupported OpenAI-compatible provider metadata: {other}"),
    }
}

#[must_use]
pub fn detect_provider_kind(model: &str) -> ProviderKind {
    if let Some(metadata) = metadata_for_model(model) {
        return metadata.provider;
    }
    if anthropic::has_auth_from_env_or_saved().unwrap_or(false) {
        return ProviderKind::Anthropic;
    }
    if openai_compat::has_api_key("OPENAI_API_KEY") {
        return ProviderKind::OpenAi;
    }
    if openai_compat::has_api_key("XAI_API_KEY") {
        return ProviderKind::Xai;
    }
    if openai_compat::has_api_key("GEMINI_API_KEY") {
        return ProviderKind::Gemini;
    }
    ProviderKind::Anthropic
}

#[must_use]
pub fn max_tokens_for_model(model: &str) -> u32 {
    model_token_limit(model).map_or_else(
        || {
            let canonical = resolve_model_alias(model);
            if canonical.contains("opus") {
                32_000
            } else {
                64_000
            }
        },
        |limit| limit.max_output_tokens,
    )
}

#[must_use]
pub fn model_token_limit(model: &str) -> Option<ModelTokenLimit> {
    let canonical = resolve_model_alias(model);
    match canonical.as_str() {
        "claude-opus-4-6" => Some(ModelTokenLimit {
            max_output_tokens: 32_000,
            context_window_tokens: 200_000,
        }),
        "claude-sonnet-4-6" | "claude-haiku-4-5-20251213" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 200_000,
        }),
        "grok-3" | "grok-3-mini" => Some(ModelTokenLimit {
            max_output_tokens: 64_000,
            context_window_tokens: 131_072,
        }),
        _ => None,
    }
}

pub fn preflight_message_request(request: &MessageRequest) -> Result<(), ApiError> {
    let Some(limit) = model_token_limit(&request.model) else {
        return Ok(());
    };

    let estimated_input_tokens = estimate_message_request_input_tokens(request);
    let estimated_total_tokens = estimated_input_tokens.saturating_add(request.max_tokens);
    if estimated_total_tokens > limit.context_window_tokens {
        return Err(ApiError::ContextWindowExceeded {
            model: resolve_model_alias(&request.model),
            estimated_input_tokens,
            requested_output_tokens: request.max_tokens,
            estimated_total_tokens,
            context_window_tokens: limit.context_window_tokens,
        });
    }

    Ok(())
}

fn configured_provider_metadata() -> Result<Vec<ProviderMetadata>, ApiError> {
    let mut configured = Vec::new();
    if anthropic::has_auth_from_env_or_saved()? {
        configured.push(anthropic_metadata());
    }
    if openai_compat::has_api_key("OPENAI_API_KEY") {
        configured.push(openai_metadata());
    }
    if openai_compat::has_api_key("XAI_API_KEY") {
        configured.push(xai_metadata());
    }
    if openai_compat::has_api_key("GEMINI_API_KEY") {
        configured.push(gemini_metadata());
    }
    if openai_compat::has_api_key("OPENAI_COMPAT_API_KEY") {
        configured.push(openai_compatible_metadata());
    }
    if anthropic::has_auth_with_env("ANTHROPIC_COMPAT_API_KEY", "ANTHROPIC_COMPAT_AUTH_TOKEN")? {
        configured.push(anthropic_compatible_metadata());
    }
    if anthropic::has_auth_with_env("OLLAMA_API_KEY", "OLLAMA_AUTH_TOKEN")?
        || env_var_present("OLLAMA_BASE_URL")
    {
        configured.push(ollama_metadata());
    }
    Ok(configured)
}

fn env_var_present(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
}

fn estimate_message_request_input_tokens(request: &MessageRequest) -> u32 {
    let mut estimate = estimate_serialized_tokens(&request.messages);
    estimate = estimate.saturating_add(estimate_serialized_tokens(&request.system));
    estimate = estimate.saturating_add(estimate_serialized_tokens(&request.tools));
    estimate = estimate.saturating_add(estimate_serialized_tokens(&request.tool_choice));
    estimate
}

fn estimate_serialized_tokens<T: Serialize>(value: &T) -> u32 {
    serde_json::to_vec(value)
        .ok()
        .map_or(0, |bytes| (bytes.len() / 4 + 1) as u32)
}

#[cfg(test)]
mod tests {
    use runtime::{set_active_provider_override, RuntimeProviderConfig};
    use serde_json::json;

    use crate::error::ApiError;
    use crate::types::{
        InputContentBlock, InputMessage, MessageRequest, ToolChoice, ToolDefinition,
    };

    use super::{
        detect_provider_kind, max_tokens_for_model, model_token_limit, preflight_message_request,
        resolve_model_alias, resolve_provider_metadata, ProviderKind,
    };

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
            detect_provider_kind("claude-sonnet-4-6"),
            ProviderKind::Anthropic
        );
        assert_eq!(detect_provider_kind("gpt-5"), ProviderKind::OpenAi);
        assert_eq!(detect_provider_kind("gemini-2.5-pro"), ProviderKind::Gemini);
    }

    #[test]
    fn explicit_provider_override_wins_over_model_family() {
        let config = RuntimeProviderConfig::default().with_id(Some("ollama".to_string()));
        let metadata =
            resolve_provider_metadata("claude-sonnet-4-6", Some(&config)).expect("provider");
        assert_eq!(metadata.id, "ollama");
    }

    #[test]
    fn process_provider_override_is_applied() {
        set_active_provider_override(Some(
            RuntimeProviderConfig::default().with_id(Some("openai-compatible".to_string())),
        ));
        let metadata = resolve_provider_metadata("custom-model", None)
            .expect("provider override should apply");
        assert_eq!(metadata.id, "openai-compatible");
        set_active_provider_override(None);
    }

    #[test]
    fn keeps_existing_max_token_heuristic() {
        assert_eq!(max_tokens_for_model("opus"), 32_000);
        assert_eq!(max_tokens_for_model("grok-3"), 64_000);
    }

    #[test]
    fn returns_context_window_metadata_for_supported_models() {
        assert_eq!(
            model_token_limit("claude-sonnet-4-6")
                .expect("claude-sonnet-4-6 should be registered")
                .context_window_tokens,
            200_000
        );
        assert_eq!(
            model_token_limit("grok-mini")
                .expect("grok-mini should resolve to a registered model")
                .context_window_tokens,
            131_072
        );
    }

    #[test]
    fn preflight_blocks_requests_that_exceed_the_model_context_window() {
        let request = MessageRequest {
            model: "claude-sonnet-4-6".to_string(),
            max_tokens: 64_000,
            messages: vec![InputMessage {
                role: "user".to_string(),
                content: vec![InputContentBlock::Text {
                    text: "x".repeat(600_000),
                }],
            }],
            system: Some("Keep the answer short.".to_string()),
            tools: Some(vec![ToolDefinition {
                name: "weather".to_string(),
                description: Some("Fetches weather".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": { "city": { "type": "string" } },
                }),
            }]),
            tool_choice: Some(ToolChoice::Auto),
            stream: true,
        };

        let error = preflight_message_request(&request)
            .expect_err("oversized request should be rejected before the provider call");

        match error {
            ApiError::ContextWindowExceeded {
                model,
                estimated_input_tokens,
                requested_output_tokens,
                estimated_total_tokens,
                context_window_tokens,
            } => {
                assert_eq!(model, "claude-sonnet-4-6");
                assert!(estimated_input_tokens > 136_000);
                assert_eq!(requested_output_tokens, 64_000);
                assert!(estimated_total_tokens > context_window_tokens);
                assert_eq!(context_window_tokens, 200_000);
            }
            other => panic!("expected context-window preflight failure, got {other:?}"),
        }
    }

    #[test]
    fn preflight_skips_unknown_models() {
        let request = MessageRequest {
            model: "unknown-model".to_string(),
            max_tokens: 64_000,
            messages: vec![InputMessage {
                role: "user".to_string(),
                content: vec![InputContentBlock::Text {
                    text: "x".repeat(600_000),
                }],
            }],
            system: None,
            tools: None,
            tool_choice: None,
            stream: false,
        };

        preflight_message_request(&request)
            .expect("models without context metadata should skip the guarded preflight");
    }
}
