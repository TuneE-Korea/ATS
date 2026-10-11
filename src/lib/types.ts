// Rust 쪽 타입(ats-core, src-tauri/src/pipeline.rs)과 1:1로 맞춘다.
// Rust에서 필드를 바꾸면 여기도 같이 바꿔야 한다. (Phase 2에서 tauri-specta로 자동 생성 고려)

export type TranslationMode = "hotkey" | "realtime";

export interface Settings {
  deeplApiKey: string;
  targetLang: string;
  hotkey: string;
  mode: TranslationMode;
  onlyTranslateKorean: boolean;
}

export interface TranslateResponse {
  text: string;
  detectedSourceLang: string | null;
}

export interface Usage {
  used: number;
  limit: number;
}

export type PipelineEvent =
  | { kind: "translated"; source: string; translated: string; detectedSourceLang: string | null }
  | { kind: "skipped"; reason: string }
  | { kind: "failed"; message: string };
