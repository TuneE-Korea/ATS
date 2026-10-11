import { hasCondition, profileFromWindow } from "../lib/profiles";
import { useConfig } from "../stores/config";
import { LanguageSelect } from "./LanguageSelect";
import { Section, inputClass } from "./Section";

const iconButton = "rounded px-2 py-1 text-xs text-zinc-500 hover:bg-zinc-100 disabled:opacity-30 dark:hover:bg-zinc-800";

export function ProfilesSection() {
  const settings = useConfig((s) => s.draft?.settings);
  const { addProfile, updateProfile, removeProfile, moveProfile } = useConfig();
  if (!settings) return null;
  const { profiles } = settings;

  return (
    <Section title="앱·사이트별 번역 언어">
      <p className="text-xs text-zinc-500">
        단축키를 누른 창이 조건에 맞으면 그 언어로 번역합니다. 위에서부터 처음 맞는 프로필을 씁니다. 브라우저는
        사이트마다 프로세스가 같으니 "창 제목에 포함"으로 구분하세요 (예: reddit, 5ch).
      </p>

      {profiles.map((p, i) => (
        <div key={i} className="space-y-2 rounded-lg border border-zinc-200 p-3 dark:border-zinc-800">
          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={p.enabled}
              onChange={(e) => updateProfile(i, { enabled: e.target.checked })}
              aria-label="사용"
            />
            <input
              className={`${inputClass} font-medium`}
              value={p.name}
              onChange={(e) => updateProfile(i, { name: e.target.value })}
              placeholder="이름"
            />
            <button className={iconButton} onClick={() => moveProfile(i, -1)} disabled={i === 0} aria-label="위로">
              ▲
            </button>
            <button
              className={iconButton}
              onClick={() => moveProfile(i, 1)}
              disabled={i === profiles.length - 1}
              aria-label="아래로"
            >
              ▼
            </button>
            <button className={`${iconButton} text-red-500`} onClick={() => removeProfile(i)} aria-label="삭제">
              ✕
            </button>
          </div>
          <div className="grid grid-cols-3 gap-2">
            <input
              className={inputClass}
              value={p.process}
              onChange={(e) => updateProfile(i, { process: e.target.value })}
              placeholder="프로세스 (Discord.exe)"
            />
            <input
              className={inputClass}
              value={p.titleContains}
              onChange={(e) => updateProfile(i, { titleContains: e.target.value })}
              placeholder="창 제목에 포함"
            />
            <LanguageSelect value={p.targetLang} onChange={(targetLang) => updateProfile(i, { targetLang })} />
          </div>
          {!hasCondition(p) && (
            <p className="text-xs text-amber-600">프로세스나 창 제목 중 하나는 입력해야 적용됩니다.</p>
          )}
        </div>
      ))}

      <button
        className="w-full rounded-md border border-dashed border-zinc-300 py-2 text-sm text-zinc-500 dark:border-zinc-700"
        onClick={() => addProfile(profileFromWindow("", settings.targetLang))}
      >
        + 프로필 추가
      </button>
    </Section>
  );
}
