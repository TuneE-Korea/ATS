import type { ProviderId } from "../lib/types";
import { PROVIDER_LABEL } from "../lib/types";
import { useConfig } from "../stores/config";
import { Field, Section, inputClass } from "./Section";

const ALL: ProviderId[] = ["deepl", "azure"];

export function ProvidersSection() {
  const draft = useConfig((s) => s.draft);
  const editSettings = useConfig((s) => s.editSettings);
  const editSecrets = useConfig((s) => s.editSecrets);
  if (!draft) return null;
  const { secrets, settings } = draft;
  const first = settings.providerOrder[0] ?? "deepl";

  return (
    <Section title="번역 서비스">
      <p className="text-xs text-zinc-500">
        키는 이 PC의 Windows 자격 증명 관리자에 저장됩니다. 먼저 쓰는 서비스가 한도 초과 등으로 실패하면 다른
        서비스로 자동 전환합니다.
      </p>

      <Field label="DeepL API 키" hint="DeepL API Free (월 50만 자). 키는 ':fx'로 끝납니다.">
        <input
          type="password"
          className={inputClass}
          value={secrets.deeplApiKey}
          onChange={(e) => editSecrets({ deeplApiKey: e.target.value })}
          placeholder="xxxxxxxx-xxxx-...:fx"
          autoComplete="off"
        />
      </Field>

      <div className="grid grid-cols-[1fr_9rem] gap-2">
        <Field label="Azure Translator 키" hint="F0 무료 티어 (월 200만 자).">
          <input
            type="password"
            className={inputClass}
            value={secrets.azureKey}
            onChange={(e) => editSecrets({ azureKey: e.target.value })}
            autoComplete="off"
          />
        </Field>
        <Field label="지역" hint="예: koreacentral">
          <input
            className={inputClass}
            value={secrets.azureRegion}
            onChange={(e) => editSecrets({ azureRegion: e.target.value })}
            placeholder="koreacentral"
          />
        </Field>
      </div>

      <Field label="먼저 사용할 서비스">
        <select
          className={inputClass}
          value={first}
          onChange={(e) => {
            const head = e.target.value as ProviderId;
            editSettings({ providerOrder: [head, ...ALL.filter((p) => p !== head)] });
          }}
        >
          {ALL.map((p) => (
            <option key={p} value={p}>
              {PROVIDER_LABEL[p]}
            </option>
          ))}
        </select>
      </Field>
    </Section>
  );
}
