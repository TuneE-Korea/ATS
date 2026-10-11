//! Windows UI Automation(접근성 API)으로 지금 포커스된 입력창의 글과 위치를 읽는다.
//!
//! 화면 낭독기가 쓰는 것과 같은 API다. Discord·Slack 같은 Electron 앱은 크롬 엔진이 접근성 정보를
//! 제공하므로 입력창 내용을 읽을 수 있다. 앱마다 제공 수준이 달라서 못 읽는 경우는 None으로 둔다.

use ats_core::desktop::ScreenRect;

pub struct FocusedField {
    pub text: String,
    pub rect: ScreenRect,
}

/// 입력창이 아주 길어도(문서 편집기 등) 이 이상은 읽지 않는다.
#[cfg(windows)]
const MAX_CHARS: i32 = 4000;

#[cfg(windows)]
pub use imp::Uia;

#[cfg(windows)]
mod imp {
    use super::{FocusedField, MAX_CHARS};
    use ats_core::desktop::ScreenRect;
    use windows::Win32::{
        System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED},
        UI::Accessibility::{
            CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTextPattern, IUIAutomationValuePattern,
            UIA_DocumentControlTypeId, UIA_EditControlTypeId, UIA_TextPatternId, UIA_ValuePatternId,
        },
    };

    /// COM 객체라 만든 스레드(감시 스레드)에서만 쓴다.
    pub struct Uia(IUIAutomation);

    impl Uia {
        pub fn new() -> windows::core::Result<Self> {
            // SAFETY: 이 스레드를 COM MTA로 초기화하고 UIA 클라이언트를 만든다. 스레드가 끝날 때까지 쓴다.
            unsafe {
                CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
                Ok(Self(CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?))
            }
        }

        /// 포커스가 글을 쓸 수 있는 입력창에 있으면 그 내용과 화면 위치. 비밀번호 칸은 읽지 않는다.
        pub fn focused_field(&self) -> Option<FocusedField> {
            // SAFETY: UIA 읽기 전용 호출. 실패는 모두 None으로 처리한다.
            unsafe {
                let el = self.0.GetFocusedElement().ok()?;
                if el.CurrentIsPassword().ok()?.as_bool() {
                    return None;
                }
                let control = el.CurrentControlType().ok()?;
                if control != UIA_EditControlTypeId && control != UIA_DocumentControlTypeId {
                    return None;
                }
                let text = read_text(&el)?;
                let r = el.CurrentBoundingRectangle().ok()?;
                Some(FocusedField { text, rect: ScreenRect { left: r.left, top: r.top, right: r.right, bottom: r.bottom } })
            }
        }
    }

    /// 단순 입력창은 Value 패턴, 리치 에디터는 Text 패턴으로 읽힌다. 둘 다 시도한다.
    unsafe fn read_text(el: &IUIAutomationElement) -> Option<String> {
        if let Ok(value) = el.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId) {
            if let Ok(v) = value.CurrentValue() {
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
        let text = el.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId).ok()?;
        Some(text.DocumentRange().ok()?.GetText(MAX_CHARS).ok()?.to_string())
    }
}
