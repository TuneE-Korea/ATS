//! 데스크톱 앱 실시간 미리보기 (Phase 4, 실험).
//!
//! 감시 스레드가 주기적으로 포커스된 입력창의 글을 읽고(UI Automation), 판단은 `ats_core::desktop::PreviewTracker`에 맡긴다.
//! 미리보기가 떠 있는 동안에만 Tab·Esc를 전역 단축키로 잡아서 적용/닫기에 쓰고, 닫히면 바로 놓아준다.

use crate::{active_window, input, overlay, state::AppState, translator};
use ats_core::{
    desktop::{Action, PreviewTracker, ScreenRect},
    ActiveWindow, ProviderId,
};
use serde::Serialize;
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut};

/// 입력창을 읽는 간격. 짧을수록 반응이 빠르지만 CPU를 더 쓴다.
const POLL: Duration = Duration::from_millis(150);
/// 기능이 꺼져 있을 때는 설정이 바뀌었는지만 가끔 본다.
const IDLE_POLL: Duration = Duration::from_secs(1);
/// 미리보기 화면이 높이를 알려주기 전에 쓰는 높이 (CSS 픽셀).
const DEFAULT_HEIGHT: f64 = 64.0;

pub const EVENT_NAME: &str = "ats://preview";

pub fn apply_key() -> Shortcut {
    Shortcut::new(None, Code::Tab)
}

pub fn dismiss_key() -> Shortcut {
    Shortcut::new(None, Code::Escape)
}

/// 미리보기 창에 보내는 내용.
#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PreviewPayload {
    Loading,
    Result { text: String, provider: ProviderId, profile: Option<String> },
    Error { message: String },
}

#[derive(Default)]
pub struct DesktopPreview {
    tracker: Mutex<PreviewTracker>,
    /// 미리보기를 붙일 입력창 위치와 미리보기 내용 높이.
    layout: Mutex<Option<(ScreenRect, f64)>>,
}

pub fn start<R: Runtime>(app: &AppHandle<R>) {
    app.manage(DesktopPreview::default());
    overlay::init(app);
    let app = app.clone();
    std::thread::Builder::new()
        .name("ats-desktop-preview".into())
        .spawn(move || watch(app))
        .expect("감시 스레드 생성 실패");
}

#[cfg(windows)]
fn watch<R: Runtime>(app: AppHandle<R>) {
    let uia = match crate::uia::Uia::new() {
        Ok(uia) => uia,
        Err(e) => return log::error!("UI Automation 초기화 실패, 데스크톱 미리보기를 끕니다: {e}"),
    };
    let started = Instant::now();
    loop {
        let settings = app.state::<AppState>().settings();
        if !(settings.mode == ats_core::TranslationMode::Realtime && settings.desktop_realtime) {
            observe(&app, None, started, 0);
            std::thread::sleep(IDLE_POLL);
            continue;
        }
        std::thread::sleep(POLL);

        let window = active_window::current();
        let field = settings.desktop_preview_enabled_for(&window.process).then(|| uia.focused_field()).flatten();
        if let Some(f) = &field {
            let preview = app.state::<DesktopPreview>();
            let mut layout = preview.layout.lock().unwrap();
            let height = layout.map_or(DEFAULT_HEIGHT, |(_, h)| h);
            *layout = Some((f.rect, height));
        }
        let action = observe(&app, field.as_ref().map(|f| f.text.as_str()), started, settings.realtime_debounce_ms);
        if let Action::Translate(source) = action {
            request(&app, source, window);
        }
    }
}

#[cfg(not(windows))]
fn watch<R: Runtime>(_app: AppHandle<R>) {}

/// 관찰 결과를 추적기에 넘기고, 숨겨야 하면 숨긴다. 번역 요청은 호출한 쪽이 한다.
fn observe<R: Runtime>(app: &AppHandle<R>, text: Option<&str>, started: Instant, debounce_ms: u32) -> Action {
    let now = started.elapsed().as_millis() as u64;
    let action = app.state::<DesktopPreview>().tracker.lock().unwrap().observe(text, now, u64::from(debounce_ms));
    if action == Action::Hide {
        hide(app);
    }
    action
}

fn request<R: Runtime>(app: &AppHandle<R>, source: String, window: ActiveWindow) {
    show(app, PreviewPayload::Loading);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let result = translator::translate(&app, &source, &window).await;
        let preview = app.state::<DesktopPreview>();
        let payload = {
            let mut tracker = preview.tracker.lock().unwrap();
            match result {
                Ok(p) if tracker.on_translated(&source, &p.text) => {
                    PreviewPayload::Result { text: p.text, provider: p.provider, profile: p.profile }
                }
                Err(message) if tracker.on_failed(&source) => PreviewPayload::Error { message },
                // 그 사이 글이 바뀌었다. 이미 숨겼으니 아무것도 안 한다.
                _ => return,
            }
        };
        let ready = matches!(payload, PreviewPayload::Result { .. });
        show(&app, payload);
        if ready {
            set_keys(&app, true);
        }
    });
}

fn show<R: Runtime>(app: &AppHandle<R>, payload: PreviewPayload) {
    let _ = app.emit_to(overlay::LABEL, EVENT_NAME, payload);
    // 락을 쥔 채로 창을 움직이지 않는다. 창 메시지를 처리하는 메인 스레드가 같은 락을 기다리면 멈춘다.
    let layout = *app.state::<DesktopPreview>().layout.lock().unwrap();
    if let Some((field, height)) = layout {
        overlay::show(app, field, height);
    }
}

fn hide<R: Runtime>(app: &AppHandle<R>) {
    set_keys(app, false);
    overlay::hide(app);
}

/// 미리보기 화면이 내용을 그린 뒤 실제 높이를 알려준다.
pub fn set_content_height<R: Runtime>(app: &AppHandle<R>, height: f64) {
    let preview = app.state::<DesktopPreview>();
    let mut layout = preview.layout.lock().unwrap();
    if let Some((field, h)) = layout.as_mut() {
        *h = height;
        let field = *field;
        drop(layout);
        if preview.tracker.lock().unwrap().is_pending_or_shown() {
            overlay::show(app, field, height);
        }
    }
}

/// Tab·Esc를 잡거나 놓는다. 미리보기가 없을 때 Tab을 잡고 있으면 다른 앱에서 Tab이 안 먹는다.
fn set_keys<R: Runtime>(app: &AppHandle<R>, on: bool) {
    let gs = app.global_shortcut();
    for key in [apply_key(), dismiss_key()] {
        let registered = gs.is_registered(key);
        let result = match (on, registered) {
            (true, false) => gs.register(key),
            (false, true) => gs.unregister(key),
            _ => Ok(()),
        };
        if let Err(e) = result {
            log::warn!("미리보기 키 {} 실패: {e}", if on { "등록" } else { "해제" });
        }
    }
}

/// Tab: 미리보기의 번역문으로 입력창 전체를 바꾼다.
pub fn apply<R: Runtime>(app: &AppHandle<R>) {
    let translated = {
        let preview = app.state::<DesktopPreview>();
        let mut tracker = preview.tracker.lock().unwrap();
        let text = tracker.applicable_now().map(str::to_owned);
        tracker.dismiss();
        text
    };
    hide(app);
    let Some(text) = translated else { return };
    tauri::async_runtime::spawn_blocking(move || {
        let backup = input::ClipboardBackup::take();
        let result = input::select_all().and_then(|()| input::paste_text(&text));
        backup.restore();
        if let Err(e) = result {
            log::warn!("미리보기 적용 실패: {e}");
        }
    });
}

/// Esc: 미리보기를 닫는다. 같은 글로는 다시 띄우지 않는다.
pub fn dismiss<R: Runtime>(app: &AppHandle<R>) {
    app.state::<DesktopPreview>().tracker.lock().unwrap().dismiss();
    hide(app);
}
