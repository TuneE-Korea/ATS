//! API 키를 Windows 자격 증명 관리자에 저장한다.
//!
//! 제어판 → 자격 증명 관리자 → Windows 자격 증명에서 "dev.tunee.ats" 항목으로 보인다.
//! 키 세 개를 JSON 하나로 묶어 한 항목에 넣는다 (항목 하나당 최대 2560바이트).

use ats_core::{SecretStore, Secrets};

#[cfg(windows)]
pub struct CredentialManagerStore {
    entry: keyring::Entry,
}

#[cfg(windows)]
impl CredentialManagerStore {
    pub fn new() -> Result<Self, String> {
        let entry = keyring::Entry::new("dev.tunee.ats", "api-keys").map_err(|e| e.to_string())?;
        Ok(Self { entry })
    }
}

#[cfg(windows)]
impl SecretStore for CredentialManagerStore {
    fn load(&self) -> Result<Secrets, String> {
        match self.entry.get_password() {
            Ok(raw) => serde_json::from_str(&raw).map_err(|e| e.to_string()),
            Err(keyring::Error::NoEntry) => Ok(Secrets::default()),
            Err(e) => Err(format!("자격 증명 관리자 읽기 실패: {e}")),
        }
    }

    fn save(&self, secrets: &Secrets) -> Result<(), String> {
        let raw = serde_json::to_string(secrets).map_err(|e| e.to_string())?;
        self.entry.set_password(&raw).map_err(|e| format!("자격 증명 관리자 저장 실패: {e}"))
    }
}

/// Windows가 아니면 메모리에만 둔다 (이 앱의 대상은 Windows라서 개발 확인용).
pub fn create() -> Result<Box<dyn SecretStore>, String> {
    #[cfg(windows)]
    {
        Ok(Box::new(CredentialManagerStore::new()?))
    }
    #[cfg(not(windows))]
    {
        Ok(Box::new(ats_core::MemorySecretStore::default()))
    }
}
