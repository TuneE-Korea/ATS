use ats_core::Settings;
use std::{path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager};

pub struct AppState {
    settings: Mutex<Settings>,
    settings_path: PathBuf,
}

impl AppState {
    pub fn load(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        // Windows: %APPDATA%\dev.tunee.ats\settings.json
        let settings_path = app.path().app_config_dir()?.join("settings.json");
        let settings = Settings::load(&settings_path)?;
        Ok(Self { settings: Mutex::new(settings), settings_path })
    }

    /// 락을 오래 잡지 않도록 복사본을 돌려준다. 번역 요청(네트워크) 중에 락을 들고 있으면 안 된다.
    pub fn settings(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn replace_settings(&self, next: Settings) -> std::io::Result<()> {
        next.save(&self.settings_path)?;
        *self.settings.lock().unwrap() = next;
        Ok(())
    }
}
