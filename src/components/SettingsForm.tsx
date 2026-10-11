import { TARGET_LANGUAGES } from "../lib/languages";
import { isDirty, useSettings } from "../stores/settings";
import { Field, Section, inputClass } from "./Section";

export function SettingsForm() {
  const { saved, draft, saving, error, edit, reset, save } = useSettings();
  if (!draft) return null;
  const dirty = isDirty(saved, draft);

  return (
    <Section title="설정">
      <Field label="DeepL API 키" hint="DeepL API Free 키는 ':fx'로 끝납니다. 이 PC의 설정 파일에만 저장됩니다.">
        <input
          type="password"
          className={inputClass}
          value={draft.deeplApiKey}
          onChange={(e) => edit({ deeplApiKey: e.target.value })}
          placeholder="xxxxxxxx-xxxx-...:fx"
          autoComplete="off"
        />
      </Field>

      <Field label="번역할 언어">
        <select className={inputClass} value={draft.targetLang} onChange={(e) => edit({ targetLang: e.target.value })}>
          {TARGET_LANGUAGES.map((l) => (
            <option key={l.code} value={l.code}>
              {l.label}
            </option>
          ))}
        </select>
      </Field>

      <Field label="단축키" hint="예: Ctrl+Alt+T, Ctrl+Shift+Space. 다른 앱과 겹치면 저장할 때 알려줍니다.">
        <input className={inputClass} value={draft.hotkey} onChange={(e) => edit({ hotkey: e.target.value })} />
      </Field>

      <Field label="입력 방식">
        <select
          className={inputClass}
          value={draft.mode}
          onChange={(e) => edit({ mode: e.target.value as typeof draft.mode })}
        >
          <option value="hotkey">단축키로 일괄 번역</option>
          <option value="realtime" disabled>
            실시간 미리보기 (준비 중)
          </option>
        </select>
      </Field>

      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={draft.onlyTranslateKorean}
          onChange={(e) => edit({ onlyTranslateKorean: e.target.checked })}
        />
        한국어가 포함된 글만 번역 (이미 번역된 글을 다시 번역하지 않음)
      </label>

      {error && <p className="text-sm text-red-600">{error}</p>}

      <div className="flex justify-end gap-2">
        <button
          className="rounded-md px-3 py-2 text-sm text-zinc-600 disabled:opacity-40"
          onClick={reset}
          disabled={!dirty || saving}
        >
          되돌리기
        </button>
        <button
          className="rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white disabled:opacity-40"
          onClick={() => void save()}
          disabled={!dirty || saving}
        >
          {saving ? "저장 중..." : "저장"}
        </button>
      </div>
    </Section>
  );
}
