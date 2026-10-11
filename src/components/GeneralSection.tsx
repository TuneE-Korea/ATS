import { useConfig } from "../stores/config";
import { LanguageSelect } from "./LanguageSelect";
import { Field, Section, inputClass } from "./Section";

export function GeneralSection() {
  const settings = useConfig((s) => s.draft?.settings);
  const edit = useConfig((s) => s.editSettings);
  if (!settings) return null;

  return (
    <Section title="기본 설정">
      <Field label="기본 번역 언어" hint="아래 프로필에 해당하지 않는 창에서 사용합니다.">
        <LanguageSelect value={settings.targetLang} onChange={(targetLang) => edit({ targetLang })} />
      </Field>

      <Field label="단축키" hint="예: Ctrl+Alt+T, Ctrl+Shift+Space. 다른 앱과 겹치면 저장할 때 알려줍니다.">
        <input className={inputClass} value={settings.hotkey} onChange={(e) => edit({ hotkey: e.target.value })} />
      </Field>

      <Field label="입력 방식">
        <select
          className={inputClass}
          value={settings.mode}
          onChange={(e) => edit({ mode: e.target.value as typeof settings.mode })}
        >
          <option value="hotkey">단축키로 일괄 번역</option>
          <option value="realtime">단축키 + 브라우저 실시간 미리보기</option>
        </select>
      </Field>

      {settings.mode === "realtime" && (
        <Field
          label="미리보기 대기 시간 (ms)"
          hint="입력을 멈추고 이만큼 지나면 번역합니다. 짧을수록 빠르지만 무료 한도를 더 씁니다."
        >
          <input
            type="number"
            min={300}
            max={3000}
            step={100}
            className={inputClass}
            value={settings.realtimeDebounceMs}
            onChange={(e) => edit({ realtimeDebounceMs: Number(e.target.value) || 600 })}
          />
        </Field>
      )}

      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={settings.onlyTranslateKorean}
          onChange={(e) => edit({ onlyTranslateKorean: e.target.checked })}
        />
        한국어가 포함된 글만 번역 (이미 번역된 글을 다시 번역하지 않음)
      </label>
    </Section>
  );
}
