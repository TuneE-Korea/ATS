//! 브라우저 확장용 로컬 서버. 라우터와 인증은 `ats_core::server`에 있고, 여기서는 실제 번역만 구현한다.

use crate::state::AppState;
use ats_core::{
    resolve_target,
    server::{self, BrowserTranslateRequest, BrowserTranslateResponse, ServerBackend, ServerStatus},
    should_translate, ActiveWindow, TranslateRequest, TranslationCache, TranslationMode, TranslationProvider,
};
use async_trait::async_trait;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Runtime};

/// 설정 창에 보여줄 서버 상태.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub port: u16,
    pub running: bool,
    pub error: Option<String>,
}

pub struct ServerInfoState(pub Mutex<ServerInfo>);

struct Backend<R: Runtime> {
    app: AppHandle<R>,
    /// 실시간 미리보기는 같은 문장을 반복해서 보내기 쉬워서 무료 한도를 아끼려고 캐시한다.
    cache: Mutex<TranslationCache<BrowserTranslateResponse>>,
}

#[async_trait]
impl<R: Runtime> ServerBackend for Backend<R> {
    fn pairing_token(&self) -> String {
        self.app.state::<AppState>().config().secrets.pairing_token
    }

    fn status(&self) -> ServerStatus {
        let s = self.app.state::<AppState>().settings();
        ServerStatus { version: env!("CARGO_PKG_VERSION"), mode: s.mode, debounce_ms: s.realtime_debounce_ms }
    }

    async fn translate(&self, req: BrowserTranslateRequest) -> Result<Option<BrowserTranslateResponse>, String> {
        let state = self.app.state::<AppState>();
        let settings = state.settings();
        // 확장이 모드를 확인하지만, 단축키 모드로 바꾼 직후 들어온 요청도 막는다.
        if settings.mode != TranslationMode::Realtime || !should_translate(&req.text, settings.only_translate_korean) {
            return Ok(None);
        }

        let source = ActiveWindow { process: String::new(), title: req.title, domain: req.domain };
        let (target_lang, profile) = resolve_target(&settings.profiles, &settings.target_lang, &source);
        if let Some(hit) = self.cache.lock().unwrap().get(&req.text, target_lang) {
            return Ok(Some(hit));
        }

        let provider = state.provider()?;
        let request = TranslateRequest { text: req.text.clone(), source_lang: None, target_lang: target_lang.to_owned() };
        let res = provider.translate(&request).await.map_err(|e| e.to_string())?;
        let out = BrowserTranslateResponse {
            text: res.text,
            provider: res.provider,
            target_lang: request.target_lang,
            profile: profile.map(str::to_owned),
        };
        self.cache.lock().unwrap().insert(&req.text, &out.target_lang, out.clone());
        Ok(Some(out))
    }
}

/// 서버를 백그라운드에서 띄운다. 포트가 이미 쓰이고 있어도 앱은 계속 동작하고, 설정 창에 오류를 보여준다.
pub fn start<R: Runtime>(app: &AppHandle<R>) {
    let port = server::DEFAULT_PORT;
    app.manage(ServerInfoState(Mutex::new(ServerInfo { port, running: false, error: None })));
    let backend = Arc::new(Backend { app: app.clone(), cache: Mutex::new(TranslationCache::new(300)) });
    let app = app.clone();

    tauri::async_runtime::spawn(async move {
        let set = |running: bool, error: Option<String>| {
            let info = app.state::<ServerInfoState>();
            let mut info = info.0.lock().unwrap();
            info.running = running;
            info.error = error;
        };
        let listener = match server::bind(port).await {
            Ok(l) => l,
            Err(e) => {
                log::error!("로컬 서버 시작 실패: {e}");
                return set(false, Some(format!("{port}번 포트를 열 수 없습니다: {e}")));
            }
        };
        set(true, None);
        if let Err(e) = server::serve(listener, backend).await {
            log::error!("로컬 서버 중단: {e}");
            set(false, Some(e.to_string()));
        }
    });
}
