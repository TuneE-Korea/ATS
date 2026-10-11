//! Azure AI Translator (F0 무료 티어: 월 200만 자, 초과 시 과금 없이 차단).
//! https://learn.microsoft.com/azure/ai-services/translator/reference/v3-0-translate

use super::{ProviderError, ProviderId, TranslateRequest, TranslateResponse, TranslationProvider, Usage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const ENDPOINT: &str = "https://api.cognitive.microsofttranslator.com/translate";

pub struct Azure {
    key: String,
    region: String,
    client: reqwest::Client,
}

impl Azure {
    pub fn new(key: &str, region: &str) -> Self {
        Self { key: key.to_owned(), region: region.to_owned(), client: reqwest::Client::new() }
    }
}

/// 설정은 DeepL 코드로 통일했으므로 Azure 코드로 바꾼다.
fn to_azure_lang(deepl: &str) -> String {
    match deepl.to_ascii_uppercase().as_str() {
        "EN-US" | "EN-GB" => "en".into(),
        "ZH-HANS" | "ZH" => "zh-Hans".into(),
        "ZH-HANT" => "zh-Hant".into(),
        // Azure의 "pt"는 브라질 포르투갈어다.
        "PT-BR" => "pt".into(),
        "PT-PT" => "pt-pt".into(),
        other => other.to_ascii_lowercase(),
    }
}

#[derive(Serialize)]
struct Item<'a> {
    #[serde(rename = "Text")]
    text: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResultItem {
    detected_language: Option<Detected>,
    translations: Vec<Translation>,
}

#[derive(Deserialize)]
struct Detected {
    language: String,
}

#[derive(Deserialize)]
struct Translation {
    text: String,
}

/// 403은 Azure에서 "무료 할당량 초과"를 뜻한다 (DeepL과 다름).
fn map_status(status: u16, body: String) -> ProviderError {
    match status {
        401 => ProviderError::Unauthorized,
        403 => ProviderError::QuotaExceeded,
        429 => ProviderError::RateLimited,
        _ => ProviderError::Api { status, message: body },
    }
}

fn parse(raw: &str) -> Result<TranslateResponse, ProviderError> {
    let bad = |message: String| ProviderError::Api { status: 200, message };
    let items: Vec<ResultItem> = serde_json::from_str(raw).map_err(|e| bad(format!("응답 해석 실패: {e}")))?;
    let item = items.into_iter().next().ok_or_else(|| bad("번역 결과가 비어 있습니다".into()))?;
    let text = item.translations.into_iter().next().ok_or_else(|| bad("번역 결과가 비어 있습니다".into()))?.text;
    Ok(TranslateResponse {
        text,
        detected_source_lang: item.detected_language.map(|d| d.language.to_ascii_uppercase()),
        provider: ProviderId::Azure,
    })
}

#[async_trait]
impl TranslationProvider for Azure {
    fn id(&self) -> ProviderId {
        ProviderId::Azure
    }

    async fn translate(&self, req: &TranslateRequest) -> Result<TranslateResponse, ProviderError> {
        let mut query = vec![("api-version", "3.0".to_owned()), ("to", to_azure_lang(&req.target_lang))];
        if let Some(src) = &req.source_lang {
            query.push(("from", to_azure_lang(src)));
        }
        let mut builder = self
            .client
            .post(ENDPOINT)
            .query(&query)
            .header("Ocp-Apim-Subscription-Key", &self.key)
            .json(&[Item { text: &req.text }]);
        if !self.region.is_empty() {
            builder = builder.header("Ocp-Apim-Subscription-Region", &self.region);
        }
        let res = builder.send().await?;
        let status = res.status().as_u16();
        let raw = res.text().await?;
        if status != 200 {
            return Err(map_status(status, raw));
        }
        parse(&raw)
    }

    /// Azure Translator는 사용량 조회 API가 없다 (Azure Portal에서만 확인 가능).
    async fn usage(&self) -> Result<Option<Usage>, ProviderError> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_deepl_codes() {
        assert_eq!(to_azure_lang("EN-US"), "en");
        assert_eq!(to_azure_lang("ZH-HANS"), "zh-Hans");
        assert_eq!(to_azure_lang("PT-BR"), "pt");
        assert_eq!(to_azure_lang("JA"), "ja");
    }

    #[test]
    fn parses_translation() {
        let raw = r#"[{"detectedLanguage":{"language":"ko","score":1.0},"translations":[{"text":"Hello","to":"en"}]}]"#;
        assert_eq!(
            parse(raw).unwrap(),
            TranslateResponse { text: "Hello".into(), detected_source_lang: Some("KO".into()), provider: ProviderId::Azure }
        );
    }

    #[test]
    fn maps_azure_status_codes() {
        assert!(matches!(map_status(401, String::new()), ProviderError::Unauthorized));
        assert!(matches!(map_status(403, String::new()), ProviderError::QuotaExceeded));
        assert!(matches!(map_status(429, String::new()), ProviderError::RateLimited));
    }
}
