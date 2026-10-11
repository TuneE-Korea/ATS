//! 포커스된 입력창의 텍스트를 클립보드를 거쳐 읽고 쓴다.
//!
//! 모든 함수는 블로킹이다 (키 입력 사이에 sleep 필요). 호출부에서 `spawn_blocking`으로 감싼다.
//! 브라우저든 Discord든 Slack이든 "텍스트를 입력할 수 있는 곳"이면 같은 방식으로 동작한다.

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::{thread::sleep, time::Duration};

/// 키 입력이 대상 앱에 반영될 때까지 기다리는 시간. 느린 앱(Electron)에서 부족하면 늘린다.
const KEY_DELAY: Duration = Duration::from_millis(60);
/// 붙여넣기가 끝나기 전에 클립보드를 원복하면 원래 내용이 붙여넣어진다.
const PASTE_SETTLE: Duration = Duration::from_millis(150);

pub type InputResult<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// 사용자가 원래 복사해 둔 클립보드 텍스트. 작업이 끝나면 되돌린다.
pub struct ClipboardBackup(Option<String>);

impl ClipboardBackup {
    pub fn take() -> Self {
        Self(Clipboard::new().ok().and_then(|mut c| c.get_text().ok()))
    }

    pub fn restore(self) {
        let Ok(mut c) = Clipboard::new() else { return };
        let _ = match self.0 {
            Some(text) => c.set_text(text),
            None => c.clear(),
        };
    }
}

/// 사용자가 아직 누르고 있을 수 있는 수정자 키를 OS에 "뗐다"고 알린다.
fn release_modifiers(enigo: &mut Enigo) -> InputResult<()> {
    for key in [Key::Control, Key::Alt, Key::Shift, Key::Meta] {
        enigo.key(key, Direction::Release).map_err(err)?;
    }
    Ok(())
}

fn ctrl_chord(enigo: &mut Enigo, c: char) -> InputResult<()> {
    enigo.key(Key::Control, Direction::Press).map_err(err)?;
    let result = enigo.key(Key::Unicode(c), Direction::Click).map_err(err);
    // 실패하더라도 Ctrl이 눌린 채로 남지 않게 항상 뗀다.
    enigo.key(Key::Control, Direction::Release).map_err(err)?;
    result?;
    sleep(KEY_DELAY);
    Ok(())
}

/// 입력창 전체를 선택해 복사한다. 복사된 것이 없으면 None.
pub fn copy_focused_text() -> InputResult<Option<String>> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(err)?;
    let mut clipboard = Clipboard::new().map_err(err)?;

    release_modifiers(&mut enigo)?;
    sleep(KEY_DELAY);

    // 비워 두지 않으면 복사가 실패했을 때 예전 클립보드 내용을 입력창 내용으로 착각한다.
    clipboard.clear().map_err(err)?;
    ctrl_chord(&mut enigo, 'a')?;
    ctrl_chord(&mut enigo, 'c')?;

    Ok(clipboard.get_text().ok().filter(|t| !t.is_empty()))
}

/// 입력창 전체를 선택한다 (미리보기 적용 전).
pub fn select_all() -> InputResult<()> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(err)?;
    release_modifiers(&mut enigo)?;
    ctrl_chord(&mut enigo, 'a')
}

/// 선택 영역(직전에 Ctrl+A 한 상태)을 `text`로 덮어쓴다.
pub fn paste_text(text: &str) -> InputResult<()> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(err)?;
    Clipboard::new().map_err(err)?.set_text(text).map_err(err)?;
    ctrl_chord(&mut enigo, 'v')?;
    sleep(PASTE_SETTLE);
    Ok(())
}
