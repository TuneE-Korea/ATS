//! ATS 데스크톱 앱. OS 레벨 작업(전역 단축키, 클립보드, 키 입력)만 담당하고
//! 번역 로직은 `ats-core`에 위임한다.

mod active_window;
mod commands;
mod hotkey;
mod input;
mod pipeline;
mod secret_store;
mod state;
mod tray;

use state::AppState;
use tauri::{Manager, WindowEvent};

pub fn run() {
    tauri::Builder::default()
        .plugin(hotkey::plugin())
        .setup(|app| {
            let state = AppState::load(app.handle())?;
            let hotkey = state.settings().hotkey;
            app.manage(state);

            if let Err(e) = hotkey::register(app.handle(), &hotkey) {
                // 단축키 충돌이 있어도 앱은 떠야 설정 창에서 바꿀 수 있다.
                log::error!("단축키 등록 실패: {e}");
            }
            tray::create(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // 창을 닫아도 트레이에 상주한다. 종료는 트레이 메뉴에서.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::translate_text,
            commands::get_usage,
        ])
        .run(tauri::generate_context!())
        .expect("ATS 실행 중 오류");
}
