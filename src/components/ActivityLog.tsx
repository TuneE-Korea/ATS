import { languageLabel } from "../lib/languages";
import { profileFromWindow } from "../lib/profiles";
import { PROVIDER_LABEL, type PipelineEvent } from "../lib/types";
import { useConfig } from "../stores/config";
import { Section } from "./Section";

export interface LoggedEvent {
  at: Date;
  event: PipelineEvent;
}

export function ActivityLog({ items }: { items: LoggedEvent[] }) {
  const addProfile = useConfig((s) => s.addProfile);
  const defaultLang = useConfig((s) => s.draft?.settings.targetLang ?? "EN-US");

  return (
    <Section title="최근 단축키 번역">
      {items.length === 0 ? (
        <p className="text-sm text-zinc-500">아직 없습니다. 아무 입력창에서 단축키를 눌러 보세요.</p>
      ) : (
        <ul className="space-y-3">
          {items.map(({ at, event }) => (
            <li key={at.getTime()} className="space-y-1 text-sm">
              <div className="flex items-center gap-2 text-xs text-zinc-500">
                <span>{at.toLocaleTimeString()}</span>
                <span className="truncate">{event.window.process || "알 수 없는 앱"}</span>
                {event.window.process && (
                  <button
                    className="ml-auto shrink-0 text-indigo-600 hover:underline"
                    onClick={() => addProfile(profileFromWindow(event.window.process, defaultLang))}
                  >
                    이 앱으로 프로필 만들기
                  </button>
                )}
              </div>
              <Body event={event} />
            </li>
          ))}
        </ul>
      )}
    </Section>
  );
}

function Body({ event: e }: { event: PipelineEvent }) {
  switch (e.kind) {
    case "translated":
      return (
        <>
          <p>
            <span className="text-zinc-500">{e.source}</span> → <span>{e.translated}</span>
          </p>
          <p className="text-xs text-zinc-500">
            {languageLabel(e.targetLang)} · {e.profile ? `프로필 "${e.profile}"` : "기본 언어"} ·{" "}
            {PROVIDER_LABEL[e.provider]}
          </p>
        </>
      );
    case "skipped":
      return <p className="text-zinc-500">{e.reason}</p>;
    case "failed":
      return <p className="text-red-600">{e.message}</p>;
  }
}
