//! 단축키 일괄 번역 파이프라인.
//!
//! 클립보드 백업 → 전체 선택·복사 → 번역 여부 판단 → 번역 → 붙여넣기 → 클립보드 원복

use crate::{input, state::AppState};
use ats_core::{build_provider, should_translate, TranslateRequest};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// 단축키 연타로 파이프라인이 겹치면 클립보드가 꼬인다. 한 번에 하나만 실행한다.
static RUNNING: AtomicBool = AtomicBool::new(false);

/// 설정 창의 "최근 작업" 목록에 표시할 이벤트.
#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PipelineEvent {
    Translated { source: String, translated: String, detected_source_lang: Option<String> },
    Skipped { reason: String },
    Failed { message: String },
}

pub const EVENT_NAME: &str = "ats://pipeline";

pub async fn run<R: Runtime>(app: &AppHandle<R>) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    let backup = input::ClipboardBackup::take();
    let event = match translate_focused(app).await {
        Ok(e) => e,
        Err(message) => PipelineEvent::Failed { message },
    };
    // 실패하든 성공하든 사용자의 클립보드는 원래대로 돌려놓는다.
    let _ = tauri::async_runtime::spawn_blocking(move || backup.restore()).await;
    RUNNING.store(false, Ordering::SeqCst);

    if let PipelineEvent::Failed { message } = &event {
        log::warn!("번역 실패: {message}");
    }
    let _ = app.emit(EVENT_NAME, event);
}

async fn translate_focused<R: Runtime>(app: &AppHandle<R>) -> Result<PipelineEvent, String> {
    let settings = app.state::<AppState>().settings();
    let provider = build_provider(&settings).map_err(|e| e.to_string())?;

    let Some(source) = blocking(input::copy_focused_text).await? else {
        return Ok(PipelineEvent::Skipped { reason: "입력창에서 텍스트를 찾지 못했습니다".into() });
    };
    if !should_translate(&source, settings.only_translate_korean) {
        return Ok(PipelineEvent::Skipped { reason: "한국어가 아니라서 건너뛰었습니다".into() });
    }

    let req = TranslateRequest { text: source.clone(), source_lang: None, target_lang: settings.target_lang };
    let res = provider.translate(&req).await.map_err(|e| e.to_string())?;

    let translated = res.text.clone();
    blocking(move || input::paste_text(&translated)).await?;

    Ok(PipelineEvent::Translated {
        source,
        translated: res.text,
        detected_source_lang: res.detected_source_lang,
    })
}

async fn blocking<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}
