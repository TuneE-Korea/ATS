//! 단축키 일괄 번역 파이프라인.
//!
//! 활성 창 확인 → 클립보드 백업 → 전체 선택·복사 → 번역 여부 판단
//! → 프로필로 목표 언어 결정 → 번역(폴백 체인) → 붙여넣기 → 클립보드 원복

use crate::{active_window, input, state::AppState};
use ats_core::{resolve_target, should_translate, ActiveWindow, ProviderId, TranslateRequest, TranslationProvider};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// 단축키 연타로 파이프라인이 겹치면 클립보드가 꼬인다. 한 번에 하나만 실행한다.
static RUNNING: AtomicBool = AtomicBool::new(false);

/// 설정 창의 "최근 작업" 목록에 표시할 이벤트.
#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum PipelineEvent {
    Translated {
        source: String,
        translated: String,
        detected_source_lang: Option<String>,
        target_lang: String,
        provider: ProviderId,
        window: ActiveWindow,
        /// 적용된 프로필 이름. 기본 언어를 썼으면 None.
        profile: Option<String>,
    },
    Skipped { reason: String, window: ActiveWindow },
    Failed { message: String, window: ActiveWindow },
}

pub const EVENT_NAME: &str = "ats://pipeline";

pub async fn run<R: Runtime>(app: &AppHandle<R>) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    // 키 입력을 보내기 전에 읽어야 한다. 그 사이 포커스가 바뀌면 엉뚱한 프로필이 적용된다.
    let window = active_window::current();
    let backup = input::ClipboardBackup::take();
    let event = match translate_focused(app, &window).await {
        Ok(e) => e,
        Err(message) => PipelineEvent::Failed { message, window },
    };
    // 실패하든 성공하든 사용자의 클립보드는 원래대로 돌려놓는다.
    let _ = tauri::async_runtime::spawn_blocking(move || backup.restore()).await;
    RUNNING.store(false, Ordering::SeqCst);

    if let PipelineEvent::Failed { message, .. } = &event {
        log::warn!("번역 실패: {message}");
    }
    let _ = app.emit(EVENT_NAME, event);
}

async fn translate_focused<R: Runtime>(app: &AppHandle<R>, window: &ActiveWindow) -> Result<PipelineEvent, String> {
    let state = app.state::<AppState>();
    let settings = state.settings();
    let provider = state.provider()?;
    let skipped = |reason: &str| Ok(PipelineEvent::Skipped { reason: reason.into(), window: window.clone() });

    let Some(source) = blocking(input::copy_focused_text).await? else {
        return skipped("입력창에서 텍스트를 찾지 못했습니다");
    };
    if !should_translate(&source, settings.only_translate_korean) {
        return skipped("한국어가 아니라서 건너뛰었습니다");
    }

    let (target_lang, profile) = resolve_target(&settings.profiles, &settings.target_lang, window);
    let req = TranslateRequest { text: source.clone(), source_lang: None, target_lang: target_lang.to_owned() };
    let res = provider.translate(&req).await.map_err(|e| e.to_string())?;

    let translated = res.text.clone();
    blocking(move || input::paste_text(&translated)).await?;

    Ok(PipelineEvent::Translated {
        source,
        translated: res.text,
        detected_source_lang: res.detected_source_lang,
        target_lang: req.target_lang,
        provider: res.provider,
        window: window.clone(),
        profile: profile.map(str::to_owned),
    })
}

async fn blocking<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}
