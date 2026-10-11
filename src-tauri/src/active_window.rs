//! 단축키를 누른 순간의 활성 창(프로세스 이름, 창 제목)을 읽는다. 프로필 선택에 쓴다.

use ats_core::ActiveWindow;

#[cfg(windows)]
pub fn current() -> ActiveWindow {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION},
        UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId},
    };

    // SAFETY: 모두 읽기 전용 Win32 호출이다. 버퍼 길이를 함께 넘기고, 연 핸들은 반드시 닫는다.
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return ActiveWindow::default();
        }

        let len = GetWindowTextLengthW(hwnd);
        let mut title_buf = vec![0u16; len as usize + 1];
        let copied = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32);
        let title = String::from_utf16_lossy(&title_buf[..copied.max(0) as usize]);

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let mut process = String::new();
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if !handle.is_null() {
            let mut buf = [0u16; 1024];
            let mut size = buf.len() as u32;
            if QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut size) != 0 {
                let path = String::from_utf16_lossy(&buf[..size as usize]);
                // 전체 경로에서 실행 파일 이름만 쓴다: C:\...\Discord.exe → Discord.exe
                process = path.rsplit('\\').next().unwrap_or_default().to_owned();
            }
            CloseHandle(handle);
        }

        ActiveWindow { process, title, domain: String::new() }
    }
}

#[cfg(not(windows))]
pub fn current() -> ActiveWindow {
    ActiveWindow::default()
}
