//! API 키 같은 비밀값.
//!
//! 저장 방식은 `SecretStore` 트레이트로 추상화한다. 앱은 Windows 자격 증명 관리자 구현을,
//! 테스트는 메모리 구현을 쓴다. 코어는 Windows API를 몰라도 된다.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Secrets {
    pub deepl_api_key: String,
    pub azure_key: String,
    /// Azure 리소스 지역 (예: koreacentral). 글로벌 리소스면 비워 둔다.
    pub azure_region: String,
}

pub trait SecretStore: Send + Sync {
    fn load(&self) -> Result<Secrets, String>;
    fn save(&self, secrets: &Secrets) -> Result<(), String>;
}

/// 테스트와 비 Windows 개발 환경용. 프로세스가 끝나면 사라진다.
#[derive(Default)]
pub struct MemorySecretStore(std::sync::Mutex<Secrets>);

impl SecretStore for MemorySecretStore {
    fn load(&self) -> Result<Secrets, String> {
        Ok(self.0.lock().unwrap().clone())
    }

    fn save(&self, secrets: &Secrets) -> Result<(), String> {
        *self.0.lock().unwrap() = secrets.clone();
        Ok(())
    }
}
