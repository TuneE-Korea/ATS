use ats_core::{build_provider, provider::FallbackChain, SecretStore, Secrets, Settings};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager};

/// 설정 창과 주고받는 묶음. 파일(settings)과 자격 증명 관리자(secrets)에 나눠 저장된다.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub settings: Settings,
    pub secrets: Secrets,
}

pub struct AppState {
    config: Mutex<Config>,
    settings_path: PathBuf,
    secret_store: Box<dyn SecretStore>,
}

impl AppState {
    pub fn load(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        // Windows: %APPDATA%\dev.tunee.ats\settings.json (수 KB라 C 드라이브에 둬도 무방)
        let settings_path = app.path().app_config_dir()?.join("settings.json");
        let secret_store = crate::secret_store::create()?;
        let mut settings = Settings::load(&settings_path)?;
        let mut secrets = secret_store.load()?;

        // Phase 1 설정 파일의 평문 키를 자격 증명 관리자로 옮기고 파일에서 지운다.
        if let Some(legacy) = settings.take_legacy_deepl_key() {
            if secrets.deepl_api_key.is_empty() {
                secrets.deepl_api_key = legacy;
                secret_store.save(&secrets)?;
            }
            settings.save(&settings_path)?;
            log::info!("DeepL 키를 자격 증명 관리자로 이전했습니다");
        }

        // 브라우저 확장 연결 토큰은 처음 실행할 때 한 번 만든다.
        if secrets.pairing_token.is_empty() {
            secrets.pairing_token = new_pairing_token();
            secret_store.save(&secrets)?;
        }

        Ok(Self { config: Mutex::new(Config { settings, secrets }), settings_path, secret_store })
    }

    /// 락을 오래 잡지 않도록 복사본을 돌려준다. 번역 요청(네트워크) 중에 락을 들고 있으면 안 된다.
    pub fn config(&self) -> Config {
        self.config.lock().unwrap().clone()
    }

    pub fn settings(&self) -> Settings {
        self.config.lock().unwrap().settings.clone()
    }

    pub fn provider(&self) -> Result<FallbackChain, String> {
        let Config { settings, secrets } = self.config();
        build_provider(&settings, &secrets).map_err(|e| e.to_string())
    }

    pub fn replace(&self, next: Config) -> Result<(), String> {
        next.settings.save(&self.settings_path).map_err(|e| format!("설정 저장 실패: {e}"))?;
        self.secret_store.save(&next.secrets)?;
        *self.config.lock().unwrap() = next;
        Ok(())
    }
}

pub fn new_pairing_token() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}
