//! 번역 공급자 추상화 (전략 패턴).
//!
//! 단축키 파이프라인과 설정 UI는 `TranslationProvider` 트레이트만 안다.
//! 공급자 추가나 폴백 체인은 이 모듈 안에서만 바뀐다.

mod azure;
mod chain;
mod deepl;

pub use azure::Azure;
pub use chain::FallbackChain;
pub use deepl::DeepL;

use crate::{Secrets, Settings};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderId {
    Deepl,
    Azure,
}

impl ProviderId {
    pub fn label(self) -> &'static str {
        match self {
            Self::Deepl => "DeepL",
            Self::Azure => "Azure",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TranslateRequest {
    pub text: String,
    /// None이면 공급자의 자동 감지를 쓴다.
    pub source_lang: Option<String>,
    /// DeepL 형식 코드 (EN-US, ZH-HANS ...). 다른 공급자는 내부에서 자기 형식으로 바꾼다.
    pub target_lang: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateResponse {
    pub text: String,
    pub detected_source_lang: Option<String>,
    /// 실제로 번역한 공급자. 폴백이 일어났는지 화면에서 확인할 수 있다.
    pub provider: ProviderId,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub provider: ProviderId,
    pub used: u64,
    pub limit: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("API 키가 설정된 번역 서비스가 없습니다")]
    MissingApiKey,
    #[error("API 키가 올바르지 않습니다")]
    Unauthorized,
    #[error("무료 사용량을 모두 썼습니다")]
    QuotaExceeded,
    #[error("요청이 너무 많습니다. 잠시 후 다시 시도하세요")]
    RateLimited,
    #[error("번역 API 오류 ({status}): {message}")]
    Api { status: u16, message: String },
    #[error("네트워크 오류: {0}")]
    Network(#[from] reqwest::Error),
    #[error("모든 번역 서비스가 실패했습니다 ({0})")]
    AllFailed(String),
}

#[async_trait]
pub trait TranslationProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    async fn translate(&self, req: &TranslateRequest) -> Result<TranslateResponse, ProviderError>;
    /// 사용량 조회를 지원하지 않는 공급자는 None.
    async fn usage(&self) -> Result<Option<Usage>, ProviderError>;
}

/// 설정의 순서대로, 키가 있는 공급자만 모아 폴백 체인을 만든다.
pub fn build_provider(settings: &Settings, secrets: &Secrets) -> Result<FallbackChain, ProviderError> {
    let providers: Vec<Box<dyn TranslationProvider>> = settings
        .provider_order
        .iter()
        .filter_map(|id| -> Option<Box<dyn TranslationProvider>> {
            match id {
                ProviderId::Deepl => {
                    let key = secrets.deepl_api_key.trim();
                    (!key.is_empty()).then(|| Box::new(DeepL::new(key)) as _)
                }
                ProviderId::Azure => {
                    let key = secrets.azure_key.trim();
                    (!key.is_empty()).then(|| Box::new(Azure::new(key, secrets.azure_region.trim())) as _)
                }
            }
        })
        .collect();
    if providers.is_empty() {
        return Err(ProviderError::MissingApiKey);
    }
    Ok(FallbackChain::new(providers))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_providers_without_keys_and_keeps_order() {
        let settings = Settings { provider_order: vec![ProviderId::Azure, ProviderId::Deepl], ..Settings::default() };
        let secrets = Secrets { deepl_api_key: "k:fx".into(), azure_key: "a".into(), ..Secrets::default() };
        assert_eq!(build_provider(&settings, &secrets).unwrap().ids(), vec![ProviderId::Azure, ProviderId::Deepl]);

        let only_deepl = Secrets { deepl_api_key: "k:fx".into(), ..Secrets::default() };
        assert_eq!(build_provider(&settings, &only_deepl).unwrap().ids(), vec![ProviderId::Deepl]);
    }

    #[test]
    fn no_keys_is_an_error() {
        assert!(matches!(
            build_provider(&Settings::default(), &Secrets::default()),
            Err(ProviderError::MissingApiKey)
        ));
    }
}
