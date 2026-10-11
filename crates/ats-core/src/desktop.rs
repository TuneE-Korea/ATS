//! 데스크톱 앱 실시간 미리보기의 판단 로직 (Phase 4).
//!
//! 앱 쪽은 일정 간격으로 "지금 포커스된 입력창의 글"을 읽어 `PreviewTracker::observe`에 넘기기만 한다.
//! 언제 번역을 요청하고, 언제 미리보기를 숨기고, Tab을 눌렀을 때 적용해도 되는지는 여기서 정한다.
//! OS와 시간에 의존하지 않아서(시각은 인자로 받음) Linux에서도 테스트할 수 있다.

use crate::detect::should_translate;

/// 이보다 짧은 글은 미리보기를 띄우지 않는다 ("ㅋㅋ"마다 요청하지 않기 위해). 확장과 같은 기준.
pub const MIN_CHARS: usize = 2;

pub fn should_preview(text: &str) -> bool {
    let t = text.trim();
    t.chars().count() >= MIN_CHARS && should_translate(t, true)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// 할 일 없음.
    Idle,
    /// 미리보기를 숨긴다 (입력창을 벗어났거나 내용이 바뀜).
    Hide,
    /// 이 글을 번역해서 미리보기를 띄운다.
    Translate(String),
}

#[derive(Debug, Default)]
pub struct PreviewTracker {
    /// 마지막으로 본 입력창 내용과 그 내용이 처음 보인 시각.
    current: Option<(String, u64)>,
    /// 번역을 요청한 원문. 같은 글로 두 번 요청하지 않는다.
    requested: Option<String>,
    /// 화면에 떠 있는 미리보기 (원문, 번역문).
    shown: Option<(String, String)>,
    /// Esc나 적용으로 닫았는지. 글이 바뀌면 풀린다.
    closed: bool,
}

impl PreviewTracker {
    /// `text`: 포커스된 입력창의 글. 대상 앱이 아니거나 입력창이 아니면 None.
    pub fn observe(&mut self, text: Option<&str>, now_ms: u64, debounce_ms: u64) -> Action {
        let Some(text) = text else {
            let was_active = self.current.is_some() || self.requested.is_some() || self.shown.is_some();
            *self = Self::default();
            return if was_active { Action::Hide } else { Action::Idle };
        };

        if self.current.as_ref().is_none_or(|(t, _)| t != text) {
            self.current = Some((text.to_owned(), now_ms));
            self.closed = false;
            // 글이 바뀌면 떠 있던 미리보기는 더 이상 맞지 않는다. 멈추면 다시 번역한다.
            let stale = self.requested.take().is_some() | self.shown.take().is_some();
            return if stale { Action::Hide } else { Action::Idle };
        }

        let changed_at = self.current.as_ref().map_or(now_ms, |(_, at)| *at);
        let settled = now_ms.saturating_sub(changed_at) >= debounce_ms;
        if settled && self.requested.as_deref() != Some(text) && should_preview(text) {
            self.requested = Some(text.to_owned());
            return Action::Translate(text.to_owned());
        }
        Action::Idle
    }

    /// 번역 결과가 왔을 때. 그 사이 글이 바뀌었으면 false (띄우지 않는다).
    pub fn on_translated(&mut self, source: &str, translated: &str) -> bool {
        let still_current = self.requested.as_deref() == Some(source)
            && self.current.as_ref().is_some_and(|(t, _)| t == source);
        if still_current {
            self.shown = Some((source.to_owned(), translated.to_owned()));
        }
        still_current
    }

    /// 번역이 실패했을 때. 같은 글로 다시 시도하지 않도록 requested는 남겨 둔다.
    pub fn on_failed(&mut self, source: &str) -> bool {
        self.requested.as_deref() == Some(source) && self.current.as_ref().is_some_and(|(t, _)| t == source)
    }

    /// Tab을 눌렀을 때 적용할 번역문. 입력창 글이 미리보기의 원문과 같을 때만 준다.
    pub fn applicable(&self, current_text: &str) -> Option<&str> {
        let (source, translated) = self.shown.as_ref()?;
        (source.trim() == current_text.trim()).then_some(translated.as_str())
    }

    /// 마지막으로 본 입력창 글 기준으로 적용할 번역문. 감시 주기 사이에 글이 바뀌었으면 다음 관찰에서 숨겨진다.
    pub fn applicable_now(&self) -> Option<&str> {
        self.applicable(&self.current.as_ref()?.0)
    }

    /// 번역을 기다리는 중이거나 미리보기가 떠 있는지 (숨긴 뒤 늦게 온 크기 조정으로 다시 띄우지 않기 위해).
    pub fn is_pending_or_shown(&self) -> bool {
        !self.closed
            && (self.shown.is_some()
                || self.requested.is_some() && self.current.as_ref().map(|(t, _)| t) == self.requested.as_ref())
    }

    pub fn is_shown(&self) -> bool {
        self.shown.is_some()
    }

    /// 미리보기를 닫는다 (Esc, 적용 후). 같은 글로 다시 띄우지 않도록 현재 글은 기억한다.
    pub fn dismiss(&mut self) {
        self.shown = None;
        self.closed = true;
    }
}

/// 화면 좌표 사각형 (물리 픽셀).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScreenRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl ScreenRect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
}

/// 미리보기 창 위치. 입력창 아래가 기본이고, 채팅 앱처럼 입력창이 화면 맨 아래면 위로 올린다.
/// 확장의 `placement.ts`와 같은 규칙이다.
pub fn place_preview(field: ScreenRect, size: (i32, i32), work_area: ScreenRect, gap: i32) -> (i32, i32) {
    let (width, height) = size;
    let max_left = (work_area.right - width - gap).max(work_area.left + gap);
    let left = field.left.clamp(work_area.left + gap, max_left);
    let below = field.bottom + gap;
    let top = if below + height <= work_area.bottom - gap {
        below
    } else {
        (field.top - gap - height).max(work_area.top + gap)
    };
    (left, top)
}

#[cfg(test)]
mod tests {
    use super::*;

    const D: u64 = 600;

    fn settle(t: &mut PreviewTracker, text: &str, start: u64) -> Action {
        assert_eq!(t.observe(Some(text), start, D), Action::Idle);
        t.observe(Some(text), start + D, D)
    }

    #[test]
    fn translates_once_after_typing_stops() {
        let mut t = PreviewTracker::default();
        assert_eq!(t.observe(Some("안녕"), 0, D), Action::Idle);
        assert_eq!(t.observe(Some("안녕하세요"), 100, D), Action::Idle);
        assert_eq!(t.observe(Some("안녕하세요"), 500, D), Action::Idle);
        assert_eq!(t.observe(Some("안녕하세요"), 700, D), Action::Translate("안녕하세요".into()));
        assert_eq!(t.observe(Some("안녕하세요"), 2000, D), Action::Idle);
    }

    #[test]
    fn ignores_non_korean_and_short_text() {
        let mut t = PreviewTracker::default();
        assert_eq!(settle(&mut t, "hello there", 0), Action::Idle);
        assert_eq!(settle(&mut t, "ㅋ", 1000), Action::Idle);
    }

    #[test]
    fn hides_when_text_changes_or_focus_leaves() {
        let mut t = PreviewTracker::default();
        assert_eq!(settle(&mut t, "안녕하세요", 0), Action::Translate("안녕하세요".into()));
        assert!(t.on_translated("안녕하세요", "Hello"));
        assert_eq!(t.observe(Some("안녕하세요 반가워요"), 700, D), Action::Hide);
        assert!(!t.is_shown());
        assert_eq!(t.observe(None, 800, D), Action::Hide);
        assert_eq!(t.observe(None, 900, D), Action::Idle);
    }

    #[test]
    fn drops_late_results_for_old_text() {
        let mut t = PreviewTracker::default();
        settle(&mut t, "안녕하세요", 0);
        t.observe(Some("안녕하세요 반가워요"), 700, D);
        assert!(!t.on_translated("안녕하세요", "Hello"));
        assert!(!t.is_shown());
    }

    #[test]
    fn applies_only_to_the_previewed_text() {
        let mut t = PreviewTracker::default();
        settle(&mut t, "안녕하세요", 0);
        t.on_translated("안녕하세요", "Hello");
        assert_eq!(t.applicable("안녕하세요 "), Some("Hello"));
        assert_eq!(t.applicable("안녕"), None);
        assert_eq!(t.applicable_now(), Some("Hello"));
        assert!(t.is_pending_or_shown());
        t.dismiss();
        assert!(!t.is_pending_or_shown());
        assert_eq!(t.applicable("안녕하세요"), None);
        // 닫은 뒤 같은 글로는 다시 띄우지 않는다.
        assert_eq!(t.observe(Some("안녕하세요"), 5000, D), Action::Idle);
    }

    #[test]
    fn places_below_or_flips_above() {
        let work = ScreenRect { left: 0, top: 0, right: 1920, bottom: 1040 };
        let field = ScreenRect { left: 300, top: 200, right: 900, bottom: 240 };
        assert_eq!(place_preview(field, (500, 100), work, 8), (300, 248));
        let chat = ScreenRect { left: 300, top: 960, right: 900, bottom: 1000 };
        assert_eq!(place_preview(chat, (500, 100), work, 8), (300, 852));
        let right_edge = ScreenRect { left: 1700, top: 200, right: 1900, bottom: 240 };
        assert_eq!(place_preview(right_edge, (500, 100), work, 8).0, 1920 - 500 - 8);
    }
}
