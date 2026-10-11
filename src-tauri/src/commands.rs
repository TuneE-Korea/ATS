//! 설정 창(React)에서 `invoke`로 호출하는 커맨드. 오류는 화면에 그대로 보여줄 문자열로 돌려준다.

use crate::{hotkey, state::AppState};
use ats_core::{build_provider, Settings, TranslateRequest, TranslateResponse, Usage};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, state: State<'_, AppState>, settings: Settings) -> Result<(), String> {
    let previous = state.settings();
    if previous.hotkey != settings.hotkey {
        // 새 단축키 등록이 실패하면 저장하지 않고, 이전 단축키를 다시 살린다.
        if let Err(e) = hotkey::register(&app, &settings.hotkey) {
            let _ = hotkey::register(&app, &previous.hotkey);
            return Err(e);
        }
    }
    state.replace_settings(settings).map_err(|e| format!("설정 저장 실패: {e}"))
}

/// 설정 창의 "번역 테스트" 칸에서 쓴다. 단축키 파이프라인과 같은 공급자를 사용한다.
#[tauri::command]
pub async fn translate_text(state: State<'_, AppState>, text: String) -> Result<TranslateResponse, String> {
    let settings = state.settings();
    let provider = build_provider(&settings).map_err(|e| e.to_string())?;
    let req = TranslateRequest { text, source_lang: None, target_lang: settings.target_lang };
    provider.translate(&req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_usage(state: State<'_, AppState>) -> Result<Option<Usage>, String> {
    let provider = build_provider(&state.settings()).map_err(|e| e.to_string())?;
    provider.usage().await.map_err(|e| e.to_string())
}
