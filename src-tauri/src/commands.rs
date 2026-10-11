//! 설정 창(React)에서 `invoke`로 호출하는 커맨드. 오류는 화면에 그대로 보여줄 문자열로 돌려준다.

use crate::{
    hotkey,
    server::{ServerInfo, ServerInfoState},
    state::{new_pairing_token, AppState, Config},
};
use ats_core::{TranslateRequest, TranslateResponse, TranslationProvider, Usage};
use tauri::{AppHandle, State};

/// 데스크톱 미리보기 창이 내용을 그린 뒤 높이(CSS 픽셀)를 알려준다.
#[tauri::command]
pub async fn preview_layout(app: AppHandle, height: f64) {
    crate::desktop_preview::set_content_height(&app, height);
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Config {
    state.config()
}

#[tauri::command]
pub fn save_config(app: AppHandle, state: State<'_, AppState>, config: Config) -> Result<(), String> {
    let previous = state.settings();
    if previous.hotkey != config.settings.hotkey {
        // 새 단축키 등록이 실패하면 저장하지 않고, 이전 단축키를 다시 살린다.
        if let Err(e) = hotkey::register(&app, &config.settings.hotkey) {
            let _ = hotkey::register(&app, &previous.hotkey);
            return Err(e);
        }
    }
    state.replace(config)
}

/// 설정 창의 "번역 테스트" 칸에서 쓴다. 단축키 파이프라인과 같은 폴백 체인을 사용한다.
#[tauri::command]
pub async fn translate_text(state: State<'_, AppState>, text: String) -> Result<TranslateResponse, String> {
    let provider = state.provider()?;
    let req = TranslateRequest { text, source_lang: None, target_lang: state.settings().target_lang };
    provider.translate(&req).await.map_err(|e| e.to_string())
}

/// 사용량 조회를 지원하는 공급자(현재 DeepL)의 사용량 목록.
#[tauri::command]
pub async fn get_usage(state: State<'_, AppState>) -> Result<Vec<Usage>, String> {
    Ok(state.provider()?.usages().await)
}

#[tauri::command]
pub fn get_server_info(info: State<'_, ServerInfoState>) -> ServerInfo {
    info.0.lock().unwrap().clone()
}

/// 토큰이 유출됐다고 생각될 때 새로 만든다. 확장 옵션에 새 토큰을 다시 넣어야 한다.
#[tauri::command]
pub fn regenerate_pairing_token(state: State<'_, AppState>) -> Result<Config, String> {
    let mut config = state.config();
    config.secrets.pairing_token = new_pairing_token();
    state.replace(config.clone())?;
    Ok(config)
}
