//! 브라우저 확장용 로컬 HTTP 서버의 라우터.
//!
//! 라우터만 코어에 두고, 실제 번역(`ServerBackend`)은 앱이 구현한다. 그래서 인증과 응답 형식을
//! Tauri 없이 테스트할 수 있다. 서버는 127.0.0.1에만 열린다.
//!
//! 보안: 아무 웹페이지나 `fetch("http://127.0.0.1:47120")`를 시도할 수 있으므로
//! 1) 확장 출처(chrome-extension://) 외의 Origin은 거부하고 2) 연결 토큰을 요구한다.

use crate::settings::TranslationMode;
use crate::ProviderId;
use async_trait::async_trait;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const DEFAULT_PORT: u16 = 47120;
pub const TOKEN_HEADER: &str = "x-ats-token";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserTranslateRequest {
    pub text: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserTranslateResponse {
    pub text: String,
    pub provider: ProviderId,
    pub target_lang: String,
    pub profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub version: &'static str,
    pub mode: TranslationMode,
    pub debounce_ms: u32,
}

#[async_trait]
pub trait ServerBackend: Send + Sync {
    fn pairing_token(&self) -> String;
    fn status(&self) -> ServerStatus;
    /// 번역하지 않을 글(한국어 아님 등)이면 Ok(None).
    async fn translate(&self, req: BrowserTranslateRequest) -> Result<Option<BrowserTranslateResponse>, String>;
}

type Backend = Arc<dyn ServerBackend>;

pub fn router(backend: Backend) -> Router {
    Router::new()
        .route("/v1/status", get(status))
        .route("/v1/translate", post(translate))
        .with_state(backend)
}

/// 127.0.0.1에만 바인딩해서 서버를 띄운다. 같은 PC의 다른 기기나 네트워크에서는 접근할 수 없다.
pub async fn bind(port: u16) -> std::io::Result<tokio::net::TcpListener> {
    tokio::net::TcpListener::bind(("127.0.0.1", port)).await
}

pub async fn serve(listener: tokio::net::TcpListener, backend: Backend) -> std::io::Result<()> {
    axum::serve(listener, router(backend)).await
}

fn is_allowed_origin(origin: &str) -> bool {
    origin.starts_with("chrome-extension://") || origin.starts_with("moz-extension://")
}

/// Origin이 없으면(확장 서비스 워커의 일부 요청, curl) 토큰만 본다. 있으면 확장 출처여야 한다.
fn authorize(headers: &HeaderMap, backend: &Backend) -> Result<(), StatusCode> {
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        if !is_allowed_origin(origin) {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    let token = backend.pairing_token();
    let given = headers.get(TOKEN_HEADER).and_then(|v| v.to_str().ok()).unwrap_or_default();
    if token.is_empty() || given != token {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(())
}

async fn status(State(backend): State<Backend>, headers: HeaderMap) -> Response {
    if let Err(code) = authorize(&headers, &backend) {
        return code.into_response();
    }
    Json(backend.status()).into_response()
}

async fn translate(
    State(backend): State<Backend>,
    headers: HeaderMap,
    Json(req): Json<BrowserTranslateRequest>,
) -> Response {
    if let Err(code) = authorize(&headers, &backend) {
        return code.into_response();
    }
    match backend.translate(req).await {
        Ok(Some(res)) => Json(res).into_response(),
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": error }))).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use tower::ServiceExt;

    struct Fake;

    #[async_trait]
    impl ServerBackend for Fake {
        fn pairing_token(&self) -> String {
            "secret".into()
        }
        fn status(&self) -> ServerStatus {
            ServerStatus { version: "test", mode: TranslationMode::Realtime, debounce_ms: 600 }
        }
        async fn translate(&self, req: BrowserTranslateRequest) -> Result<Option<BrowserTranslateResponse>, String> {
            match req.text.as_str() {
                "skip" => Ok(None),
                "boom" => Err("DeepL: 한도 초과".into()),
                _ => Ok(Some(BrowserTranslateResponse {
                    text: format!("[{}@{}]", req.text, req.domain),
                    provider: ProviderId::Deepl,
                    target_lang: "EN-US".into(),
                    profile: None,
                })),
            }
        }
    }

    fn request(origin: Option<&str>, token: Option<&str>, text: &str) -> Request<Body> {
        let mut b = Request::post("/v1/translate").header("content-type", "application/json");
        if let Some(o) = origin {
            b = b.header("origin", o);
        }
        if let Some(t) = token {
            b = b.header(TOKEN_HEADER, t);
        }
        b.body(Body::from(format!(r#"{{"text":"{text}","domain":"reddit.com"}}"#))).unwrap()
    }

    async fn send(req: Request<Body>) -> (StatusCode, String) {
        let res = router(Arc::new(Fake)).oneshot(req).await.unwrap();
        let status = res.status();
        let body = to_bytes(res.into_body(), 64 * 1024).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn translates_for_extension_with_token() {
        let (status, body) = send(request(Some("chrome-extension://abc"), Some("secret"), "안녕")).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("[안녕@reddit.com]"), "{body}");
    }

    #[tokio::test]
    async fn rejects_web_page_origin_even_with_token() {
        let (status, _) = send(request(Some("https://evil.example"), Some("secret"), "안녕")).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn rejects_missing_or_wrong_token() {
        assert_eq!(send(request(None, None, "안녕")).await.0, StatusCode::UNAUTHORIZED);
        assert_eq!(send(request(None, Some("nope"), "안녕")).await.0, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn skip_is_no_content_and_errors_are_bad_gateway() {
        assert_eq!(send(request(None, Some("secret"), "skip")).await.0, StatusCode::NO_CONTENT);
        let (status, body) = send(request(None, Some("secret"), "boom")).await;
        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert!(body.contains("한도 초과"));
    }

    #[tokio::test]
    async fn status_requires_token() {
        let req = |t: &str| Request::get("/v1/status").header(TOKEN_HEADER, t).body(Body::empty()).unwrap();
        assert_eq!(router(Arc::new(Fake)).oneshot(req("nope")).await.unwrap().status(), StatusCode::UNAUTHORIZED);
        assert_eq!(router(Arc::new(Fake)).oneshot(req("secret")).await.unwrap().status(), StatusCode::OK);
    }
}
