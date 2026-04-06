use crate::error::ApiError;
use crate::prompt_cache::{PromptCache, PromptCacheRecord, PromptCacheStats};
use crate::providers::anthropic::{self, AnthropicClient, AuthSource};
use crate::providers::openai_compat::{self, OpenAiCompatClient, OpenAiCompatConfig};
use crate::providers::{self, ProviderKind, ProviderTransport};
use crate::types::{MessageRequest, MessageResponse, StreamEvent};
use runtime::RuntimeProviderConfig;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum ProviderClient {
    Anthropic(AnthropicClient),
    AnthropicCompat(AnthropicClient),
    Ollama(AnthropicClient),
    Xai(OpenAiCompatClient),
    OpenAi(OpenAiCompatClient),
    Gemini(OpenAiCompatClient),
    OpenAiCompat(OpenAiCompatClient),
}

impl ProviderClient {
    pub fn from_model(model: &str) -> Result<Self, ApiError> {
        Self::from_model_with_provider_config(model, None)
    }

    pub fn from_model_with_anthropic_auth(
        model: &str,
        anthropic_auth: Option<AuthSource>,
    ) -> Result<Self, ApiError> {
        Self::from_model_with_provider_config_and_auth(model, None, anthropic_auth)
    }

    pub fn from_model_with_provider_config(
        model: &str,
        provider_config: Option<&RuntimeProviderConfig>,
    ) -> Result<Self, ApiError> {
        Self::from_model_with_provider_config_and_auth(model, provider_config, None)
    }

    pub fn from_model_with_provider_config_and_auth(
        model: &str,
        provider_config: Option<&RuntimeProviderConfig>,
        anthropic_auth: Option<AuthSource>,
    ) -> Result<Self, ApiError> {
        let resolved_model = providers::resolve_model_alias(model);
        let metadata = providers::resolve_provider_metadata(&resolved_model, provider_config)?;
        let base_url = providers::base_url_for_provider(metadata, provider_config)?;
        match metadata.transport {
            ProviderTransport::Anthropic => {
                let auth = anthropic_auth.unwrap_or(anthropic::auth_source_from_config(
                    providers::anthropic_config_for_metadata(metadata),
                )?);
                let client = AnthropicClient::from_auth(auth)
                    .with_base_url(base_url)
                    .with_request_profile(providers::anthropic_request_profile_for_metadata(
                        metadata,
                    ));
                Ok(match metadata.provider {
                    ProviderKind::Anthropic => Self::Anthropic(client),
                    ProviderKind::AnthropicCompatible => Self::AnthropicCompat(client),
                    ProviderKind::Ollama => Self::Ollama(client),
                    other => panic!("unsupported anthropic transport provider kind: {other:?}"),
                })
            }
            ProviderTransport::OpenAiCompat => {
                let config = providers::openai_compat_config_for_metadata(metadata);
                let client = OpenAiCompatClient::new(openai_compat::read_api_key(config)?, config)
                    .with_base_url(base_url);
                Ok(match metadata.provider {
                    ProviderKind::Xai => Self::Xai(client),
                    ProviderKind::OpenAi => Self::OpenAi(client),
                    ProviderKind::Gemini => Self::Gemini(client),
                    ProviderKind::OpenAiCompatible => Self::OpenAiCompat(client),
                    other => panic!("unsupported OpenAI-compatible provider kind: {other:?}"),
                })
            }
        }
    }

    #[must_use]
    pub const fn provider_kind(&self) -> ProviderKind {
        match self {
            Self::Anthropic(_) => ProviderKind::Anthropic,
            Self::AnthropicCompat(_) => ProviderKind::AnthropicCompatible,
            Self::Ollama(_) => ProviderKind::Ollama,
            Self::Xai(_) => ProviderKind::Xai,
            Self::OpenAi(_) => ProviderKind::OpenAi,
            Self::Gemini(_) => ProviderKind::Gemini,
            Self::OpenAiCompat(_) => ProviderKind::OpenAiCompatible,
        }
    }

    #[must_use]
    pub fn with_prompt_cache(self, prompt_cache: PromptCache) -> Self {
        match self {
            Self::Anthropic(client) => Self::Anthropic(client.with_prompt_cache(prompt_cache)),
            Self::AnthropicCompat(client) => {
                Self::AnthropicCompat(client.with_prompt_cache(prompt_cache))
            }
            Self::Ollama(client) => Self::Ollama(client.with_prompt_cache(prompt_cache)),
            other => other,
        }
    }

    #[must_use]
    pub fn prompt_cache_stats(&self) -> Option<PromptCacheStats> {
        match self {
            Self::Anthropic(client) | Self::AnthropicCompat(client) | Self::Ollama(client) => {
                client.prompt_cache_stats()
            }
            Self::Xai(_) | Self::OpenAi(_) | Self::Gemini(_) | Self::OpenAiCompat(_) => None,
        }
    }

    #[must_use]
    pub fn take_last_prompt_cache_record(&self) -> Option<PromptCacheRecord> {
        match self {
            Self::Anthropic(client) | Self::AnthropicCompat(client) | Self::Ollama(client) => {
                client.take_last_prompt_cache_record()
            }
            Self::Xai(_) | Self::OpenAi(_) | Self::Gemini(_) | Self::OpenAiCompat(_) => None,
        }
    }

    pub async fn send_message(
        &self,
        request: &MessageRequest,
    ) -> Result<MessageResponse, ApiError> {
        match self {
            Self::Anthropic(client) | Self::AnthropicCompat(client) | Self::Ollama(client) => {
                client.send_message(request).await
            }
            Self::Xai(client)
            | Self::OpenAi(client)
            | Self::Gemini(client)
            | Self::OpenAiCompat(client) => client.send_message(request).await,
        }
    }

    pub async fn stream_message(
        &self,
        request: &MessageRequest,
    ) -> Result<MessageStream, ApiError> {
        match self {
            Self::Anthropic(client) | Self::AnthropicCompat(client) | Self::Ollama(client) => {
                client
                    .stream_message(request)
                    .await
                    .map(MessageStream::Anthropic)
            }
            Self::Xai(client)
            | Self::OpenAi(client)
            | Self::Gemini(client)
            | Self::OpenAiCompat(client) => client
                .stream_message(request)
                .await
                .map(MessageStream::OpenAiCompat),
        }
    }
}

#[derive(Debug)]
pub enum MessageStream {
    Anthropic(anthropic::MessageStream),
    OpenAiCompat(openai_compat::MessageStream),
}

impl MessageStream {
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::Anthropic(stream) => stream.request_id(),
            Self::OpenAiCompat(stream) => stream.request_id(),
        }
    }

    pub async fn next_event(&mut self) -> Result<Option<StreamEvent>, ApiError> {
        match self {
            Self::Anthropic(stream) => stream.next_event().await,
            Self::OpenAiCompat(stream) => stream.next_event().await,
        }
    }
}

pub use anthropic::{
    oauth_token_is_expired, resolve_saved_oauth_token, resolve_startup_auth_source, OAuthTokenSet,
};
#[must_use]
pub fn read_base_url() -> String {
    anthropic::read_base_url()
}

#[must_use]
pub fn read_xai_base_url() -> String {
    openai_compat::read_base_url(OpenAiCompatConfig::xai())
        .unwrap_or_else(|_| openai_compat::DEFAULT_XAI_BASE_URL.to_string())
}

#[cfg(test)]
mod tests {
    use crate::providers::{detect_provider_kind, resolve_model_alias, ProviderKind};

    #[test]
    fn resolves_existing_and_grok_aliases() {
        assert_eq!(resolve_model_alias("opus"), "claude-opus-4-6");
        assert_eq!(resolve_model_alias("grok"), "grok-3");
        assert_eq!(resolve_model_alias("grok-mini"), "grok-3-mini");
    }

    #[test]
    fn provider_detection_prefers_model_family() {
        assert_eq!(detect_provider_kind("grok-3"), ProviderKind::Xai);
        assert_eq!(detect_provider_kind("gpt-5"), ProviderKind::OpenAi);
        assert_eq!(
            detect_provider_kind("claude-sonnet-4-6"),
            ProviderKind::Anthropic
        );
    }
}
