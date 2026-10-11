use tauri::{plugin::TauriPlugin, AppHandle, Runtime, Wry};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            // Pressed가 아니라 Released에서 실행한다. 누르는 순간 실행하면 사용자가 아직 쥐고 있는
            // Ctrl+Alt와 우리가 보내는 Ctrl+A가 섞여 Ctrl+Alt+A가 되어 버린다.
            if event.state() == ShortcutState::Released {
                let app = app.clone();
                tauri::async_runtime::spawn(async move { crate::pipeline::run(&app).await });
            }
        })
        .build()
}

/// 기존 단축키를 모두 해제하고 새 단축키를 등록한다.
pub fn register<R: Runtime>(app: &AppHandle<R>, hotkey: &str) -> Result<(), String> {
    let shortcut = parse(hotkey)?;
    let gs = app.global_shortcut();
    gs.unregister_all().map_err(|e| e.to_string())?;
    gs.register(shortcut).map_err(|e| format!("'{hotkey}' 등록 실패 (다른 앱이 사용 중일 수 있음): {e}"))
}

pub fn parse(hotkey: &str) -> Result<Shortcut, String> {
    hotkey.parse::<Shortcut>().map_err(|e| format!("단축키 형식 오류 '{hotkey}': {e}"))
}
