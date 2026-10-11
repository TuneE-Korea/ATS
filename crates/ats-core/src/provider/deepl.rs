//! DeepL API 공급자. https://developers.deepl.com/docs/api-reference/translate

use super::{ProviderError, ProviderId, TranslateRequest, TranslateResponse, TranslationProvider, Usage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const FREE_BASE_URL: &str = "https://api-free.deepl.com";
const PRO_BASE_URL: &str = "https://api.deepl.com";

pub struct DeepL {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl DeepL {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_owned(),
            base_url: base_url_for_key(api_key).to_owned(),
            client: reqwest::Client::new(),
        }
    }

    fn auth_header(&self) -> String {
        format!("DeepL-Auth-Key {}", self.api_key)
    }
}

/// Free 키는 `:fx`로 끝나고 엔드포인트가 다르다. 잘못된 엔드포인트로 보내면 403이 난다.
fn base_url_for_key(key: &str) -> &'static str {
    if key.ends_with(":fx") {
        FREE_BASE_URL
    } else {
        PRO_BASE_URL
    }
}

#[derive(Serialize)]
struct TranslateBody<'a> {
    text: [&'a str; 1],
    target_lang: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_lang: Option<&'a str>,
}

#[derive(Deserialize)]
struct TranslateResult {
    translations: Vec<Translation>,
}

#[derive(Deserialize)]
struct Translation {
    detected_source_language: Option<String>,
    text: String,
}

#[derive(Deserialize)]
struct UsageResult {
    character_count: u64,
    character_limit: u64,
}

/// HTTP 상태 코드를 도메인 오류로 바꾼다. 456은 DeepL 고유의 "할당량 초과" 코드다.
fn map_status(status: u16, body: String) -> ProviderError {
    match status {
        401 | 403 => ProviderError::Unauthorized,
        429 => ProviderError::RateLimited,
        456 => ProviderError::QuotaExceeded,
        _ => ProviderError::Api { status, message: body },
    }
}

fn parse_translate(raw: &str) -> Result<TranslateResponse, ProviderError> {
    let parsed: TranslateResult = serde_json::from_str(raw).map_err(|e| ProviderError::Api {
        status: 200,
        message: format!("응답 해석 실패: {e}"),
    })?;
    let first = parsed.translations.into_iter().next().ok_or(ProviderError::Api {
        status: 200,
        message: "번역 결과가 비어 있습니다".into(),
    })?;
    Ok(TranslateResponse {
        text: first.text,
        detected_source_lang: first.detected_source_language,
        provider: ProviderId::Deepl,
    })
}

#[async_trait]
impl TranslationProvider for DeepL {
    fn id(&self) -> ProviderId {
        ProviderId::Deepl
    }

    async fn translate(&self, req: &TranslateRequest) -> Result<TranslateResponse, ProviderError> {
        let body = TranslateBody {
            text: [&req.text],
            target_lang: &req.target_lang,
            source_lang: req.source_lang.as_deref(),
        };
        let res = self
            .client
            .post(format!("{}/v2/translate", self.base_url))
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;
        let status = res.status().as_u16();
        let raw = res.text().await?;
        if status != 200 {
            return Err(map_status(status, raw));
        }
        parse_translate(&raw)
    }

    async fn usage(&self) -> Result<Option<Usage>, ProviderError> {
        let res = self
            .client
            .get(format!("{}/v2/usage", self.base_url))
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        let status = res.status().as_u16();
        if status != 200 {
            return Err(map_status(status, res.text().await?));
        }
        let u: UsageResult = res.json().await?;
        Ok(Some(Usage { provider: ProviderId::Deepl, used: u.character_count, limit: u.character_limit }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_key_uses_free_endpoint() {
        assert_eq!(base_url_for_key("1234-abcd:fx"), FREE_BASE_URL);
        assert_eq!(base_url_for_key("1234-abcd"), PRO_BASE_URL);
    }

    #[test]
    fn body_omits_missing_source_lang() {
        let body = TranslateBody { text: ["안녕"], target_lang: "EN-US", source_lang: None };
        assert_eq!(
            serde_json::to_value(&body).unwrap(),
            serde_json::json!({ "text": ["안녕"], "target_lang": "EN-US" })
        );
    }

    #[test]
    fn parses_translation() {
        let raw = r#"{"translations":[{"detected_source_language":"KO","text":"Hello"}]}"#;
        assert_eq!(
            parse_translate(raw).unwrap(),
            TranslateResponse {
                text: "Hello".into(),
                detected_source_lang: Some("KO".into()),
                provider: ProviderId::Deepl
            }
        );
    }

    #[test]
    fn empty_translations_is_error() {
        assert!(parse_translate(r#"{"translations":[]}"#).is_err());
    }

    #[test]
    fn maps_deepl_status_codes() {
        assert!(matches!(map_status(403, String::new()), ProviderError::Unauthorized));
        assert!(matches!(map_status(456, String::new()), ProviderError::QuotaExceeded));
        assert!(matches!(map_status(429, String::new()), ProviderError::RateLimited));
        assert!(matches!(map_status(500, "x".into()), ProviderError::Api { status: 500, .. }));
    }
}
