//! 사용자 설정. 앱 설정 폴더의 `settings.json`에 저장한다.

use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum TranslationMode {
    /// 단축키를 누르면 입력창 전체를 번역해서 치환한다.
    #[default]
    Hotkey,
    /// 입력 중 미리보기를 띄운다. Phase 3에서 구현 예정.
    Realtime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// DeepL API 키. Free 키는 `:fx`로 끝난다.
    pub deepl_api_key: String,
    /// DeepL 목표 언어 코드 (EN-US, JA, ZH-HANS ...).
    pub target_lang: String,
    /// Tauri global-shortcut 형식의 단축키 문자열.
    pub hotkey: String,
    pub mode: TranslationMode,
    /// 한국어로 보이는 텍스트만 번역할지 여부 (이중 번역 방지).
    pub only_translate_korean: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            deepl_api_key: String::new(),
            target_lang: "EN-US".into(),
            hotkey: "Ctrl+Alt+T".into(),
            mode: TranslationMode::Hotkey,
            only_translate_korean: true,
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
            deepl_api_key: "abc:fx".into(),
            target_lang: "JA".into(),
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
    }
}
