import { useEffect, useState } from "react";
import { formatAppList, parseAppList } from "../lib/apps";
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
          <option value="realtime">단축키 + 실시간 미리보기</option>
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

      {settings.mode === "realtime" && (
        <>
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={settings.desktopRealtime}
              onChange={(e) => edit({ desktopRealtime: e.target.checked })}
            />
            데스크톱 앱에서도 미리보기 <span className="rounded bg-amber-100 px-1.5 text-xs text-amber-800">실험</span>
          </label>
          {settings.desktopRealtime && (
            <Field
              label="미리보기를 띄울 앱"
              hint="실행 파일 이름을 쉼표로 구분합니다. 아래 '최근 단축키 번역'의 '이 앱에서 미리보기' 버튼으로도 추가할 수 있습니다."
            >
              <AppListInput apps={settings.desktopRealtimeApps} onChange={(desktopRealtimeApps) => edit({ desktopRealtimeApps })} />
            </Field>
          )}
        </>
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

/** 입력 중에는 글자 그대로 두고, 칸을 벗어날 때 목록으로 정리한다 (쉼표를 치는 순간 지워지지 않게). */
function AppListInput({ apps, onChange }: { apps: string[]; onChange: (apps: string[]) => void }) {
  const [text, setText] = useState(() => formatAppList(apps));
  // "이 앱에서 미리보기" 버튼처럼 밖에서 목록이 바뀌면 칸에도 반영한다. 입력 중인 글과 같은 목록이면 그대로 둔다.
  useEffect(() => {
    setText((current) => (formatAppList(parseAppList(current)) === formatAppList(apps) ? current : formatAppList(apps)));
  }, [apps]);
  return (
    <input
      className={inputClass}
      value={text}
      placeholder="Discord.exe, slack.exe"
      onChange={(e) => {
        setText(e.target.value);
        onChange(parseAppList(e.target.value));
      }}
      onBlur={() => setText(formatAppList(apps))}
    />
  );
}
