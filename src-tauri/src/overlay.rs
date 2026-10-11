//! 데스크톱 앱 위에 띄우는 미리보기 창. tauri.conf.json의 "preview" 창(투명, 테두리 없음)을 쓴다.
//!
//! 이 창이 포커스를 가져가면 사용자가 쓰던 입력창에서 포커스가 빠져 버린다.
//! 그래서 Tauri의 show() 대신 Win32로 "활성화하지 않고" 띄운다.

use ats_core::desktop::{place_preview, ScreenRect};
use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

pub const LABEL: &str = "preview";

/// 미리보기 창 너비 (CSS 픽셀). 입력창 너비를 따르되 이 범위로 자른다.
const MIN_WIDTH: f64 = 320.0;
const MAX_WIDTH: f64 = 560.0;
const GAP: f64 = 6.0;

fn window<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window(LABEL)
}

/// 앱 시작 시 한 번: 클릭이 통과하고, 작업 표시줄·Alt+Tab에 안 나오고, 눌러도 활성화되지 않게 한다.
pub fn init<R: Runtime>(app: &AppHandle<R>) {
    let Some(w) = window(app) else { return };
    let _ = w.set_ignore_cursor_events(true);
    #[cfg(windows)]
    if let Ok(hwnd) = w.hwnd() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        };
        let hwnd = hwnd.0 as windows_sys::Win32::Foundation::HWND;
        // SAFETY: 우리 창의 확장 스타일에 플래그만 더한다.
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | (WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW) as isize);
        }
    }
}

/// 입력창 근처에 띄운다. `content_height`는 미리보기 화면이 알려준 내용 높이 (CSS 픽셀).
pub fn show<R: Runtime>(app: &AppHandle<R>, field: ScreenRect, content_height: f64) {
    let Some(w) = window(app) else { return };
    let scale = w.scale_factor().unwrap_or(1.0);
    let width = (f64::from(field.width())).clamp(MIN_WIDTH * scale, MAX_WIDTH * scale).round() as i32;
    let height = (content_height * scale).round() as i32;
    let (x, y) = place_preview(field, (width, height), work_area(field), (GAP * scale).round() as i32);

    #[cfg(windows)]
    if let Ok(hwnd) = w.hwnd() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_SHOWWINDOW,
        };
        // SAFETY: 우리 창의 위치·크기만 바꾼다. SWP_NOACTIVATE로 사용자 입력창의 포커스를 유지한다.
        unsafe {
            SetWindowPos(hwnd.0 as _, HWND_TOPMOST, x, y, width, height, SWP_NOACTIVATE | SWP_SHOWWINDOW);
        }
    }
    #[cfg(not(windows))]
    let _ = (x, y);
}

/// show()를 Win32로 했으므로 숨기기도 Win32로 한다 (Tauri는 이 창이 이미 숨겨진 줄 알고 무시할 수 있다).
pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    let Some(w) = window(app) else { return };
    #[cfg(windows)]
    if let Ok(hwnd) = w.hwnd() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
        // SAFETY: 우리 창을 숨기기만 한다.
        unsafe { ShowWindow(hwnd.0 as _, SW_HIDE) };
        return;
    }
    let _ = w.hide();
}

/// 입력창이 있는 모니터의 작업 영역 (작업 표시줄 제외).
#[cfg(windows)]
fn work_area(field: ScreenRect) -> ScreenRect {
    use windows_sys::Win32::{
        Foundation::RECT,
        Graphics::Gdi::{GetMonitorInfoW, MonitorFromRect, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    };
    let rect = RECT { left: field.left, top: field.top, right: field.right, bottom: field.bottom };
    // SAFETY: 읽기 전용 호출. cbSize를 채워서 넘긴다.
    unsafe {
        let monitor = MonitorFromRect(&rect, MONITOR_DEFAULTTONEAREST);
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut info) != 0 {
            let r = info.rcWork;
            return ScreenRect { left: r.left, top: r.top, right: r.right, bottom: r.bottom };
        }
    }
    ScreenRect { left: 0, top: 0, right: 1920, bottom: 1080 }
}

#[cfg(not(windows))]
fn work_area(_: ScreenRect) -> ScreenRect {
    ScreenRect { left: 0, top: 0, right: 1920, bottom: 1080 }
}
