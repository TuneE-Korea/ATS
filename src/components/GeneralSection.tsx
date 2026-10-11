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
          <option value="realtime" disabled>
            실시간 미리보기 (준비 중)
          </option>
        </select>
      </Field>

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
