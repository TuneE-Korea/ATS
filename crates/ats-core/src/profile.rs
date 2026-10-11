//! 앱/창별 목표 언어 프로필.
//!
//! 번역 요청이 온 곳(활성 창의 프로세스·제목, 또는 브라우저 확장이 알려준 도메인)으로 프로필을 고른다.
//! 단축키 번역은 도메인을 모르므로 브라우저 사이트는 창 제목(= 탭 제목)으로 구분하고,
//! 확장의 실시간 미리보기는 도메인으로 정확히 구분할 수 있다.

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
    /// 사이트 도메인 (예: reddit.com). 하위 도메인도 포함한다. 브라우저 확장 요청에만 일치한다.
    pub domain: String,
    pub target_lang: String,
    pub enabled: bool,
}

/// 번역 요청이 온 곳. 단축키는 활성 창의 프로세스·제목을, 브라우저 확장은 제목·도메인을 채운다.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveWindow {
    pub process: String,
    pub title: String,
    pub domain: String,
}

/// `host`가 `domain`이거나 그 하위 도메인인지. "notreddit.com"이 "reddit.com"에 걸리지 않게 점 경계를 본다.
fn domain_matches(host: &str, domain: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    let domain = domain.trim().trim_start_matches("*.").trim_start_matches('.').to_ascii_lowercase();
    host == domain || host.ends_with(&format!(".{domain}"))
}

impl Profile {
    /// 대소문자를 무시하고 비교한다. 조건이 하나도 없는 프로필은 아무 창에도 일치하지 않는다
    /// (실수로 빈 프로필을 맨 위에 두면 모든 번역이 그 언어로 바뀌는 사고를 막기 위해).
    pub fn matches(&self, window: &ActiveWindow) -> bool {
        let process = self.process.trim();
        let title = self.title_contains.trim();
        let domain = self.domain.trim();
        if !self.enabled || (process.is_empty() && title.is_empty() && domain.is_empty()) {
            return false;
        }
        let process_ok = process.is_empty() || window.process.eq_ignore_ascii_case(process);
        let title_ok = title.is_empty() || window.title.to_lowercase().contains(&title.to_lowercase());
        let domain_ok = domain.is_empty() || domain_matches(&window.domain, domain);
        process_ok && title_ok && domain_ok
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
        ActiveWindow { process: process.into(), title: title.into(), domain: String::new() }
    }

    fn browser(domain: &str, title: &str) -> ActiveWindow {
        ActiveWindow { process: String::new(), title: title.into(), domain: domain.into() }
    }

    fn profile(name: &str, process: &str, title: &str, lang: &str) -> Profile {
        Profile {
            name: name.into(),
            process: process.into(),
            title_contains: title.into(),
            target_lang: lang.into(),
            enabled: true,
            ..Profile::default()
        }
    }

    fn domain_profile(domain: &str, lang: &str) -> Profile {
        Profile { name: domain.into(), domain: domain.into(), target_lang: lang.into(), enabled: true, ..Profile::default() }
    }

    #[test]
    fn domain_matches_subdomains_on_dot_boundary() {
        let p = domain_profile("reddit.com", "EN-US");
        assert!(p.matches(&browser("reddit.com", "")));
        assert!(p.matches(&browser("old.reddit.com", "")));
        assert!(!p.matches(&browser("notreddit.com", "")));
        assert!(domain_profile("*.5ch.net", "JA").matches(&browser("egg.5ch.net", "")));
    }

    #[test]
    fn domain_profile_never_matches_hotkey_requests() {
        assert!(!domain_profile("reddit.com", "EN-US").matches(&win("chrome.exe", "Reddit - Chrome")));
    }

    #[test]
    fn title_profile_matches_browser_requests_too() {
        let p = profile("Reddit", "", "reddit", "EN-US");
        assert!(p.matches(&browser("www.reddit.com", "r/rust - Reddit")));
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
