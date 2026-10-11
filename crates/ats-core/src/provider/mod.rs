//! 번역 공급자 추상화 (전략 패턴).
//!
//! 단축키 파이프라인과 설정 UI는 이 트레이트만 안다. 공급자를 추가하거나 폴백 체인을
//! 만들 때(Phase 2) 호출하는 쪽은 바뀌지 않는다.

mod deepl;

pub use deepl::DeepL;

use crate::Settings;
use async_trait::async_trait;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct TranslateRequest {
    pub text: String,
    /// None이면 공급자의 자동 감지를 쓴다.
    pub source_lang: Option<String>,
    pub target_lang: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateResponse {
    pub text: String,
    pub detected_source_lang: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub used: u64,
    pub limit: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("API 키가 설정되지 않았습니다")]
    MissingApiKey,
    #[error("API 키가 올바르지 않습니다")]
    Unauthorized,
    #[error("이번 달 무료 사용량을 모두 썼습니다")]
    QuotaExceeded,
    #[error("요청이 너무 많습니다. 잠시 후 다시 시도하세요")]
    RateLimited,
    #[error("번역 API 오류 ({status}): {message}")]
    Api { status: u16, message: String },
    #[error("네트워크 오류: {0}")]
    Network(#[from] reqwest::Error),
}

#[async_trait]
pub trait TranslationProvider: Send + Sync {
    fn id(&self) -> &'static str;
    async fn translate(&self, req: &TranslateRequest) -> Result<TranslateResponse, ProviderError>;
    /// 사용량 조회를 지원하지 않는 공급자는 None.
    async fn usage(&self) -> Result<Option<Usage>, ProviderError>;
}

/// 현재 설정으로 공급자를 만든다. Phase 2에서 공급자 선택과 폴백 체인이 여기로 들어온다.
pub fn build_provider(settings: &Settings) -> Result<Box<dyn TranslationProvider>, ProviderError> {
    let key = settings.deepl_api_key.trim();
    if key.is_empty() {
        return Err(ProviderError::MissingApiKey);
    }
    Ok(Box::new(DeepL::new(key)))
}
