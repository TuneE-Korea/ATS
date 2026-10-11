import { useState } from "react";
import { errorMessage, ipc } from "../lib/ipc";
import type { TranslateResponse } from "../lib/types";
import { Section, inputClass } from "./Section";

// 저장된 설정(API 키, 목표 언어)으로 번역이 되는지 단축키 없이 확인하는 칸.
export function TranslateTester() {
  const [text, setText] = useState("안녕하세요, 이 글 정말 유익하네요!");
  const [result, setResult] = useState<TranslateResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  async function run() {
    setPending(true);
    setError(null);
    try {
      setResult(await ipc.translateText(text));
    } catch (e) {
      setResult(null);
      setError(errorMessage(e));
    } finally {
      setPending(false);
    }
  }

  return (
    <Section title="번역 테스트">
      <textarea className={`${inputClass} h-20 resize-none`} value={text} onChange={(e) => setText(e.target.value)} />
      <button
        className="rounded-md border border-zinc-300 px-3 py-1.5 text-sm disabled:opacity-40 dark:border-zinc-700"
        onClick={() => void run()}
        disabled={pending || !text.trim()}
      >
        {pending ? "번역 중..." : "번역해 보기"}
      </button>
      {result && (
        <p className="rounded-md bg-zinc-100 p-3 text-sm dark:bg-zinc-800">
          {result.text}
          {result.detectedSourceLang && (
            <span className="ml-2 text-xs text-zinc-500">감지: {result.detectedSourceLang}</span>
          )}
        </p>
      )}
      {error && <p className="text-sm text-red-600">{error}</p>}
    </Section>
  );
}
