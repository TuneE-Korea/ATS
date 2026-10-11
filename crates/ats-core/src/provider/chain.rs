//! 폴백 체인. 앞 공급자가 실패하면 다음 공급자로 넘어간다.
//!
//! 체인 자체도 `TranslationProvider`를 구현한다 (컴포지트 패턴). 그래서 파이프라인은
//! 공급자가 하나인지 여럿인지 신경 쓰지 않는다.

use super::{ProviderError, ProviderId, TranslateRequest, TranslateResponse, TranslationProvider, Usage};
use async_trait::async_trait;

pub struct FallbackChain {
    providers: Vec<Box<dyn TranslationProvider>>,
}

impl FallbackChain {
    pub fn new(providers: Vec<Box<dyn TranslationProvider>>) -> Self {
        Self { providers }
    }

    pub fn ids(&self) -> Vec<ProviderId> {
        self.providers.iter().map(|p| p.id()).collect()
    }

    /// 사용량 조회를 지원하는 모든 공급자의 사용량. 조회에 실패한 공급자는 건너뛴다.
    pub async fn usages(&self) -> Vec<Usage> {
        let mut out = Vec::new();
        for p in &self.providers {
            if let Ok(Some(u)) = p.usage().await {
                out.push(u);
            }
        }
        out
    }
}

#[async_trait]
impl TranslationProvider for FallbackChain {
    fn id(&self) -> ProviderId {
        // 체인의 대표 id는 첫 번째 공급자. build_provider가 빈 체인을 만들지 않는다.
        self.providers[0].id()
    }

    async fn translate(&self, req: &TranslateRequest) -> Result<TranslateResponse, ProviderError> {
        let mut failures = Vec::new();
        for p in &self.providers {
            match p.translate(req).await {
                Ok(res) => return Ok(res),
                Err(e) => failures.push(format!("{}: {e}", p.id().label())),
            }
        }
        // 공급자가 하나뿐이면 원래 오류 문구가 더 읽기 좋지만, 이미 문자열로 바꿨으므로 그대로 합친다.
        Err(ProviderError::AllFailed(failures.join(" / ")))
    }

    async fn usage(&self) -> Result<Option<Usage>, ProviderError> {
        Ok(self.usages().await.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// 정해진 결과를 돌려주고 호출 횟수를 센다.
    struct Fake {
        id: ProviderId,
        fail: bool,
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TranslationProvider for Fake {
        fn id(&self) -> ProviderId {
            self.id
        }
        async fn translate(&self, req: &TranslateRequest) -> Result<TranslateResponse, ProviderError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                Err(ProviderError::QuotaExceeded)
            } else {
                Ok(TranslateResponse { text: format!("[{}]", req.text), detected_source_lang: None, provider: self.id })
            }
        }
        async fn usage(&self) -> Result<Option<Usage>, ProviderError> {
            Ok(None)
        }
    }

    fn fake(id: ProviderId, fail: bool) -> (Box<dyn TranslationProvider>, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        (Box::new(Fake { id, fail, calls: calls.clone() }), calls)
    }

    fn req() -> TranslateRequest {
        TranslateRequest { text: "안녕".into(), source_lang: None, target_lang: "EN-US".into() }
    }

    #[tokio::test]
    async fn uses_first_provider_when_it_succeeds() {
        let (a, a_calls) = fake(ProviderId::Deepl, false);
        let (b, b_calls) = fake(ProviderId::Azure, false);
        let res = FallbackChain::new(vec![a, b]).translate(&req()).await.unwrap();
        assert_eq!(res.provider, ProviderId::Deepl);
        assert_eq!((a_calls.load(Ordering::SeqCst), b_calls.load(Ordering::SeqCst)), (1, 0));
    }

    #[tokio::test]
    async fn falls_back_when_first_fails() {
        let (a, _) = fake(ProviderId::Deepl, true);
        let (b, _) = fake(ProviderId::Azure, false);
        let res = FallbackChain::new(vec![a, b]).translate(&req()).await.unwrap();
        assert_eq!(res.provider, ProviderId::Azure);
        assert_eq!(res.text, "[안녕]");
    }

    #[tokio::test]
    async fn reports_every_failure() {
        let (a, _) = fake(ProviderId::Deepl, true);
        let (b, _) = fake(ProviderId::Azure, true);
        let err = FallbackChain::new(vec![a, b]).translate(&req()).await.unwrap_err().to_string();
        assert!(err.contains("DeepL") && err.contains("Azure"), "{err}");
    }
}
