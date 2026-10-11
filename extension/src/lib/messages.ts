// content script ↔ 서비스 워커 ↔ ATS 앱(로컬 서버) 사이의 메시지 타입.
// 앱 쪽 응답 형식은 crates/ats-core/src/server.rs와 맞춘다.

export type Mode = "hotkey" | "realtime";

export interface ServerStatus {
  version: string;
  mode: Mode;
  debounceMs: number;
}

export interface TranslateResult {
  text: string;
  provider: "deepl" | "azure";
  targetLang: string;
  profile: string | null;
}

/** offline: 앱이 꺼져 있음, unauthorized: 토큰 불일치, failed: 번역 서비스 오류 */
export type Failure = { ok: false; reason: "offline" | "unauthorized" | "failed"; message: string };

export type Request =
  | { type: "status" }
  | { type: "translate"; text: string; domain: string; title: string }
  | { type: "check"; token: string };

export type StatusResponse = { ok: true; status: ServerStatus } | Failure;
export type TranslateResponse = { ok: true; result: TranslateResult | null } | Failure;

export const PROVIDER_LABEL = { deepl: "DeepL", azure: "Azure" } as const;
