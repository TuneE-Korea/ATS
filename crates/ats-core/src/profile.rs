//! 앱/창별 목표 언어 프로필.
//!
//! 단축키를 누른 순간의 활성 창(프로세스 이름, 창 제목)으로 프로필을 고른다.
//! 브라우저는 프로세스가 하나라서 사이트 구분은 창 제목(= 탭 제목)으로 한다.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    /// 화면 표시용 이름.
    pub name: String,
    /// 프로세스 실행 파일 이름 (예: discord.exe). 비우면 검사하지 않는다.
    pub process: String,
    /// 창 제목에 포함되어야 하는 문자열 (예: reddit). 비우면 검사하지 않는다.
    pub title_contains: String,
    pub target_lang: String,
    pub enabled: bool,
}

/// 단축키를 누른 순간의 활성 창 정보.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveWindow {
    pub process: String,
    pub title: String,
}

impl Profile {
    /// 대소문자를 무시하고 비교한다. 조건이 하나도 없는 프로필은 아무 창에도 일치하지 않는다
    /// (실수로 빈 프로필을 맨 위에 두면 모든 번역이 그 언어로 바뀌는 사고를 막기 위해).
    pub fn matches(&self, window: &ActiveWindow) -> bool {
        let process = self.process.trim();
        let title = self.title_contains.trim();
        if !self.enabled || (process.is_empty() && title.is_empty()) {
            return false;
        }
        let process_ok = process.is_empty() || window.process.eq_ignore_ascii_case(process);
        let title_ok = title.is_empty() || window.title.to_lowercase().contains(&title.to_lowercase());
        process_ok && title_ok
    }
}

/// 목표 언어와, 적용된 프로필 이름(없으면 None)을 고른다.
pub fn resolve_target<'a>(
    profiles: &'a [Profile],
    default_lang: &'a str,
    window: &ActiveWindow,
) -> (&'a str, Option<&'a str>) {
    match profiles.iter().find(|p| p.matches(window)) {
        Some(p) => (&p.target_lang, Some(&p.name)),
        None => (default_lang, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win(process: &str, title: &str) -> ActiveWindow {
        ActiveWindow { process: process.into(), title: title.into() }
    }

    fn profile(name: &str, process: &str, title: &str, lang: &str) -> Profile {
        Profile {
            name: name.into(),
            process: process.into(),
            title_contains: title.into(),
            target_lang: lang.into(),
            enabled: true,
        }
    }

    #[test]
    fn matches_process_case_insensitively() {
        let p = profile("Discord", "discord.exe", "", "JA");
        assert!(p.matches(&win("Discord.exe", "#general - Discord")));
    }

    #[test]
    fn matches_browser_tab_by_title() {
        let p = profile("Reddit", "chrome.exe", "reddit", "EN-US");
        assert!(p.matches(&win("chrome.exe", "Some post : r/rust - Reddit - Chrome")));
        assert!(!p.matches(&win("msedge.exe", "Some post - Reddit")));
    }

    #[test]
    fn empty_conditions_never_match() {
        assert!(!profile("빈 프로필", "", "  ", "JA").matches(&win("a.exe", "b")));
    }

    #[test]
    fn disabled_profile_is_skipped() {
        let mut p = profile("Discord", "discord.exe", "", "JA");
        p.enabled = false;
        assert!(!p.matches(&win("discord.exe", "")));
    }

    #[test]
    fn first_match_wins_and_falls_back_to_default() {
        let profiles = vec![
            profile("5ch", "chrome.exe", "5ch", "JA"),
            profile("Chrome", "chrome.exe", "", "EN-US"),
        ];
        assert_eq!(resolve_target(&profiles, "DE", &win("chrome.exe", "5ch.net")), ("JA", Some("5ch")));
        assert_eq!(resolve_target(&profiles, "DE", &win("chrome.exe", "Google")), ("EN-US", Some("Chrome")));
        assert_eq!(resolve_target(&profiles, "DE", &win("slack.exe", "")), ("DE", None));
    }
}
