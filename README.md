# ATS (Auto Translate Server)

입력창에 한국어로 쓴 글을 단축키 한 번으로 번역해서 바꿔 주는 Windows 트레이 앱입니다.
브라우저, Discord, Slack 등 텍스트를 입력할 수 있는 곳이면 어디서나 동작합니다.

> 로컬 전용 개인 도구입니다. 배포는 고려하지 않습니다.

## 동작 방식 (Phase 1: 단축키 일괄 번역)

1. 입력창에 한국어로 글을 쓴다.
2. `Ctrl+Alt+T` (설정에서 변경 가능)를 누른다.
3. ATS가 클립보드를 백업하고 → `Ctrl+A`, `Ctrl+C`로 입력창 내용을 읽고 → DeepL로 번역한 뒤 → `Ctrl+V`로 덮어쓰고 → 클립보드를 원래대로 돌려놓는다.

이미 외국어인 글은 건너뜁니다 (이중 번역 방지, 설정에서 끌 수 있음).

## 앱·사이트별 번역 언어 (Phase 2)

단축키를 누른 순간의 활성 창으로 번역 언어를 고릅니다. 설정 창의 "앱·사이트별 번역 언어"에서 프로필을 추가하세요.

| 예시 | 프로세스 | 창 제목에 포함 | 언어 |
|---|---|---|---|
| Discord 서버 | `Discord.exe` | | 일본어 |
| 크롬의 Reddit 탭 | `chrome.exe` | `reddit` | 영어 |

- 위에서부터 처음 맞는 프로필을 쓰고, 없으면 기본 번역 언어를 씁니다.
- 프로세스 이름을 모르면 아무 데서나 단축키를 한 번 누른 뒤, "최근 단축키 번역"의 **이 앱으로 프로필 만들기**를 누르세요.

## 번역 서비스와 자동 전환

| 서비스 | 무료 한도 | 초과 시 |
|---|---|---|
| DeepL API Free | 월 50만 자 | 차단 (과금 없음) |
| Azure Translator F0 | 월 200만 자 | 차단 (과금 없음) |

두 키를 모두 넣으면 "먼저 사용할 서비스"가 실패(한도 초과, 키 오류, 네트워크 등)할 때 다른 서비스로 자동 전환합니다.
어느 서비스가 번역했는지는 "최근 단축키 번역"에 표시됩니다.

**Azure 키 발급:** [Azure Portal](https://portal.azure.com)에서 "Translator" 리소스를 만들고 가격 책정 계층을 **F0 (Free)** 로 선택합니다.
리소스의 "키 및 엔드포인트"에서 키 1과 위치/지역(예: `koreacentral`)을 복사해 넣으세요.

API 키는 설정 파일이 아니라 **Windows 자격 증명 관리자**에 저장됩니다 (제어판 → 자격 증명 관리자 → Windows 자격 증명 → `dev.tunee.ats`).
Phase 1에서 설정 파일에 저장했던 DeepL 키는 첫 실행 때 자동으로 옮겨지고 파일에서 지워집니다.

## 처음 실행하기 (Windows)

### 1. 준비물

| 도구 | 설치 |
|---|---|
| Node.js 22 이상 | https://nodejs.org |
| Rust (stable) | https://rustup.rs |
| Microsoft C++ Build Tools | Visual Studio Installer에서 "C++를 사용한 데스크톱 개발" 워크로드 |
| WebView2 | Windows 10/11에는 기본 설치되어 있음 |

자세한 내용: https://v2.tauri.app/start/prerequisites/

### 2. DeepL API Free 키 발급

https://www.deepl.com/pro-api 에서 **DeepL API Free**에 가입합니다 (월 50만 자 무료, 초과해도 과금되지 않고 차단됨).
본인 확인용으로 카드 등록이 필요합니다. 발급된 키는 `:fx`로 끝납니다.

### 3. 실행

```powershell
npm install
npm run tauri dev
```

설정 창이 뜨면 API 키를 입력하고 저장한 뒤, "번역 테스트"로 동작을 확인하세요.
창을 닫아도 트레이에서 계속 동작합니다. 완전히 끄려면 트레이 아이콘 우클릭 → 종료.

설치 파일이 필요하면 `npm run tauri build` (결과물: `target/release/bundle/nsis/`).

## 프로젝트 구조

```
crates/ats-core/     번역 코어 (Tauri 의존성 없음, Linux에서도 테스트 가능)
  src/detect.rs        한글 비율로 번역 여부 판단
  src/profile.rs       활성 창 → 프로필 → 목표 언어 결정
  src/provider/        TranslationProvider 트레이트, DeepL·Azure 구현, 폴백 체인
  src/secrets.rs       API 키 묶음과 SecretStore 트레이트
  src/settings.rs      설정 로드/저장 (%APPDATA%\dev.tunee.ats\settings.json, 키 제외)
src-tauri/           데스크톱 앱 (OS 레벨 작업만)
  src/hotkey.rs        전역 단축키 등록
  src/input.rs         클립보드 백업/복원, 키 입력 시뮬레이션
  src/active_window.rs 활성 창의 프로세스 이름·제목 조회 (Win32)
  src/secret_store.rs  Windows 자격 증명 관리자 저장소
  src/pipeline.rs      단축키 → 복사 → 번역 → 붙여넣기 흐름
  src/commands.rs      설정 창에서 호출하는 커맨드
  src/tray.rs          트레이 아이콘과 메뉴
src/                 설정 창 (React + TS + Tailwind + Zustand)
docs/architecture.md 아키텍처 제안서와 로드맵
docs/icon.svg        앱 아이콘 원본 (tray.svg는 트레이용으로 말풍선을 키운 버전)
```

## 테스트

```powershell
npm test                    # 프론트엔드 단위 테스트 (vitest)
npm run test:core           # Rust 번역 코어 테스트
```

## 설계 메모

- **코어와 앱 분리**: 번역, 언어 감지, 설정은 `ats-core`에만 있습니다. Phase 3의 브라우저 확장용 로컬 서버도 같은 코어를 씁니다.
- **공급자 추상화**: 파이프라인은 `TranslationProvider` 트레이트만 압니다. 폴백 체인(`FallbackChain`)도 같은 트레이트를 구현해서, 공급자가 하나든 여럿이든 호출하는 쪽 코드는 같습니다.
- **비밀값 분리**: 키는 `SecretStore` 트레이트 뒤에 있어서 코어는 Windows API를 모릅니다. 테스트는 메모리 저장소를 씁니다.
- **활성 창은 키 입력 전에 읽기**: 키 입력을 보낸 뒤 읽으면 포커스가 바뀌어 엉뚱한 프로필이 적용될 수 있습니다.
- **단축키는 뗄 때 실행**: 누르는 순간 실행하면 사용자가 쥐고 있는 `Ctrl+Alt`와 우리가 보내는 `Ctrl+A`가 섞입니다.
- **TLS는 OS 기본(SChannel)**: reqwest 기본값(rustls + aws-lc)은 Windows에서 추가 C 빌드 도구가 필요해서 `native-tls`를 씁니다.

## 로드맵

- [x] Phase 1: 단축키 일괄 번역 (DeepL Free)
- [x] Phase 2: 앱/사이트별 목표 언어, 공급자 폴백 (Azure Translator F0), API 키 자격 증명 관리자 보관
- [ ] Phase 3: 브라우저 확장 + 로컬 서버, 실시간 번역 미리보기
- [ ] Phase 4 (실험): 데스크톱 앱 실시간 미리보기 (UI Automation)

## 알려진 한계

- 브라우저 사이트 구분은 창 제목(탭 제목) 기준이라, 제목에 사이트 이름이 없는 페이지는 구분되지 않습니다 (Phase 3 확장에서 도메인 기준으로 개선 예정).
- 일부 앱은 `Ctrl+A`가 입력창이 아니라 화면 전체를 선택합니다. 그런 앱에서는 아직 동작하지 않을 수 있습니다.
- 느린 앱에서 붙여넣기가 누락되면 `src-tauri/src/input.rs`의 `KEY_DELAY`, `PASTE_SETTLE`를 늘려 보세요.
