//! 사용자 설정. 앱 설정 폴더의 `settings.json`에 저장한다.
//!
//! API 키 같은 비밀값은 여기 두지 않는다 (`secrets.rs` 참고). 설정 파일은 평문이라
//! 백업이나 공유 과정에서 키가 함께 새어 나갈 수 있기 때문이다.

use crate::profile::Profile;
use crate::provider::ProviderId;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum TranslationMode {
    /// 단축키를 누르면 입력창 전체를 번역해서 치환한다.
    #[default]
    Hotkey,
    /// 단축키에 더해, 입력 중 번역 미리보기를 띄운다 (브라우저 확장, 켜 두면 데스크톱 앱도).
    Realtime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// 어떤 프로필에도 해당하지 않을 때 쓰는 목표 언어 (DeepL 코드: EN-US, JA, ZH-HANS ...).
    pub target_lang: String,
    /// Tauri global-shortcut 형식의 단축키 문자열.
    pub hotkey: String,
    pub mode: TranslationMode,
    /// 한국어로 보이는 텍스트만 번역할지 여부 (이중 번역 방지).
    pub only_translate_korean: bool,
    /// 브라우저 확장의 실시간 미리보기가 입력을 멈춘 뒤 기다리는 시간.
    pub realtime_debounce_ms: u32,
    /// 데스크톱 앱(Discord, Slack 등)에서도 실시간 미리보기를 띄울지. 실험 기능이라 기본은 끔.
    pub desktop_realtime: bool,
    /// 데스크톱 실시간 미리보기를 켤 앱의 실행 파일 이름. 대소문자 무시.
    pub desktop_realtime_apps: Vec<String>,
    /// 공급자 시도 순서. 앞의 공급자가 한도 초과 등으로 실패하면 다음 공급자로 넘어간다.
    pub provider_order: Vec<ProviderId>,
    /// 앱/창별 목표 언어. 위에서부터 처음 일치하는 프로필을 쓴다.
    pub profiles: Vec<Profile>,

    /// Phase 1 설정 파일에 평문으로 저장되던 키. 읽기만 하고 다시 쓰지 않는다 (자격 증명 저장소로 이전).
    #[serde(rename = "deeplApiKey", skip_serializing)]
    pub legacy_deepl_api_key: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            target_lang: "EN-US".into(),
            hotkey: "Ctrl+Alt+T".into(),
            mode: TranslationMode::Hotkey,
            only_translate_korean: true,
            realtime_debounce_ms: 600,
            desktop_realtime: false,
            desktop_realtime_apps: ["Discord.exe", "slack.exe", "KakaoTalk.exe", "Telegram.exe", "LINE.exe"].map(String::from).to_vec(),
            provider_order: vec![ProviderId::Deepl, ProviderId::Azure],
            profiles: Vec::new(),
            legacy_deepl_api_key: None,
        }
    }
}

impl Settings {
    /// 파일이 없으면 기본값을 돌려준다. 필드가 빠진 예전 파일도 `#[serde(default)]` 덕분에 읽힌다.
    pub fn load(path: &Path) -> io::Result<Self> {
        match fs::read_to_string(path) {
            Ok(raw) => serde_json::from_str(&raw).map_err(io::Error::other),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e),
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, raw)
    }

    /// 이 프로세스에서 데스크톱 실시간 미리보기를 띄울지.
    pub fn desktop_preview_enabled_for(&self, process: &str) -> bool {
        self.mode == TranslationMode::Realtime
            && self.desktop_realtime
            && !process.is_empty()
            && self.desktop_realtime_apps.iter().any(|app| app.trim().eq_ignore_ascii_case(process))
    }

    /// 예전 평문 키를 꺼낸다. 호출한 쪽이 비밀 저장소에 옮긴 뒤 `save`하면 파일에서 사라진다.
    pub fn take_legacy_deepl_key(&mut self) -> Option<String> {
        self.legacy_deepl_api_key.take().filter(|k| !k.trim().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let s = Settings::load(&dir.path().join("nope.json")).unwrap();
        assert_eq!(s, Settings::default());
    }

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/settings.json");
        let s = Settings {
            target_lang: "JA".into(),
            profiles: vec![Profile { name: "Discord".into(), process: "discord.exe".into(), ..Profile::default() }],
            ..Settings::default()
        };
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap(), s);
    }

    #[test]
    fn partial_file_fills_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, r#"{ "targetLang": "JA" }"#).unwrap();
        let s = Settings::load(&path).unwrap();
        assert_eq!(s.target_lang, "JA");
        assert_eq!(s.hotkey, "Ctrl+Alt+T");
        assert_eq!(s.provider_order, vec![ProviderId::Deepl, ProviderId::Azure]);
    }

    #[test]
    fn legacy_key_is_read_but_never_written_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, r#"{ "deeplApiKey": "old:fx", "targetLang": "JA" }"#).unwrap();

        let mut s = Settings::load(&path).unwrap();
        assert_eq!(s.take_legacy_deepl_key().as_deref(), Some("old:fx"));
        s.save(&path).unwrap();

        assert!(!fs::read_to_string(&path).unwrap().contains("old:fx"));
    }

    #[test]
    fn desktop_preview_needs_realtime_toggle_and_listed_app() {
        let mut s = Settings { desktop_realtime: true, ..Settings::default() };
        assert!(!s.desktop_preview_enabled_for("Discord.exe"), "hotkey mode");
        s.mode = TranslationMode::Realtime;
        assert!(s.desktop_preview_enabled_for("discord.EXE"));
        assert!(!s.desktop_preview_enabled_for("chrome.exe"));
        assert!(!s.desktop_preview_enabled_for(""));
        s.desktop_realtime = false;
        assert!(!s.desktop_preview_enabled_for("Discord.exe"));
    }
}
