// Tauri 커맨드 호출을 한 곳에 모은다. 컴포넌트는 invoke 문자열을 직접 쓰지 않는다.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { PipelineEvent, Settings, TranslateResponse, Usage } from "./types";

export const ipc = {
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<void>("save_settings", { settings }),
  translateText: (text: string) => invoke<TranslateResponse>("translate_text", { text }),
  getUsage: () => invoke<Usage | null>("get_usage"),
};

export function onPipelineEvent(handler: (e: PipelineEvent) => void): Promise<UnlistenFn> {
  return listen<PipelineEvent>("ats://pipeline", (e) => handler(e.payload));
}

/** Rust 커맨드는 오류를 문자열로 reject한다. 그 외 예외도 화면용 문자열로 바꾼다. */
export function errorMessage(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : String(e);
}
