//! 미리보기용 번역 (브라우저 확장, 데스크톱 앱 공통). 프로필로 목표 언어를 고르고, 결과를 캐시한다.

use crate::state::AppState;
use ats_core::{
    resolve_target, server::BrowserTranslateResponse, ActiveWindow, TranslateRequest, TranslationCache,
    TranslationProvider,
};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime};

/// 실시간 미리보기는 같은 문장을 반복해서 보내기 쉬워서, 무료 한도를 아끼려고 캐시한다.
pub struct PreviewCache(Mutex<TranslationCache<Preview>>);

pub type Preview = BrowserTranslateResponse;

impl Default for PreviewCache {
    fn default() -> Self {
        Self(Mutex::new(TranslationCache::new(300)))
    }
}

pub async fn translate<R: Runtime>(app: &AppHandle<R>, text: &str, source: &ActiveWindow) -> Result<Preview, String> {
    let state = app.state::<AppState>();
    let cache = app.state::<PreviewCache>();
    let settings = state.settings();
    let (target_lang, profile) = resolve_target(&settings.profiles, &settings.target_lang, source);
    if let Some(hit) = cache.0.lock().unwrap().get(text, target_lang) {
        return Ok(hit);
    }

    let provider = state.provider()?;
    let request = TranslateRequest { text: text.to_owned(), source_lang: None, target_lang: target_lang.to_owned() };
    let res = provider.translate(&request).await.map_err(|e| e.to_string())?;
    let out = Preview {
        text: res.text,
        provider: res.provider,
        target_lang: request.target_lang,
        profile: profile.map(str::to_owned),
    };
    cache.0.lock().unwrap().insert(text, &out.target_lang, out.clone());
    Ok(out)
}
