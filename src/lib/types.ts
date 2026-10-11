// Rust 쪽 타입(ats-core, src-tauri/src/state.rs, pipeline.rs)과 1:1로 맞춘다.
// Rust에서 필드를 바꾸면 여기도 같이 바꿔야 한다.

export type TranslationMode = "hotkey" | "realtime";
export type ProviderId = "deepl" | "azure";

export interface Profile {
  name: string;
  /** 실행 파일 이름. 예: Discord.exe */
  process: string;
  /** 창 제목에 포함될 문자열. 브라우저 사이트 구분용. */
  titleContains: string;
  targetLang: string;
  enabled: boolean;
}

export interface Settings {
  targetLang: string;
  hotkey: string;
  mode: TranslationMode;
  onlyTranslateKorean: boolean;
  providerOrder: ProviderId[];
  profiles: Profile[];
}

/** Windows 자격 증명 관리자에 저장되는 값. */
export interface Secrets {
  deeplApiKey: string;
  azureKey: string;
  azureRegion: string;
}

export interface Config {
  settings: Settings;
  secrets: Secrets;
}

export interface ActiveWindow {
  process: string;
  title: string;
}

export interface TranslateResponse {
  text: string;
  detectedSourceLang: string | null;
  provider: ProviderId;
}

export interface Usage {
  provider: ProviderId;
  used: number;
  limit: number;
}

export type PipelineEvent =
  | {
      kind: "translated";
      source: string;
      translated: string;
      detectedSourceLang: string | null;
      targetLang: string;
      provider: ProviderId;
      window: ActiveWindow;
      profile: string | null;
    }
  | { kind: "skipped"; reason: string; window: ActiveWindow }
  | { kind: "failed"; message: string; window: ActiveWindow };

export const PROVIDER_LABEL: Record<ProviderId, string> = { deepl: "DeepL", azure: "Azure" };
