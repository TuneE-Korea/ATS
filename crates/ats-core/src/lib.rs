//! ATS 번역 코어.
//!
//! Tauri(OS 입력 제어)와 분리된 순수 로직만 둔다. 그래서 Linux CI에서도
//! `cargo test -p ats-core`로 검증할 수 있고, Phase 3의 로컬 서버도 이 크레이트를 재사용한다.

pub mod detect;
pub mod provider;
pub mod settings;

pub use detect::should_translate;
pub use provider::{
    build_provider, ProviderError, TranslateRequest, TranslateResponse, TranslationProvider, Usage,
};
pub use settings::{Settings, TranslationMode};
