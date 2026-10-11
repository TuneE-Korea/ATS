import type { PipelineEvent } from "../lib/types";
import { Section } from "./Section";

export interface LoggedEvent {
  at: Date;
  event: PipelineEvent;
}

export function ActivityLog({ items }: { items: LoggedEvent[] }) {
  return (
    <Section title="최근 단축키 번역">
      {items.length === 0 ? (
        <p className="text-sm text-zinc-500">아직 없습니다. 아무 입력창에서 단축키를 눌러 보세요.</p>
      ) : (
        <ul className="space-y-2">
          {items.map(({ at, event }) => (
            <li key={at.getTime()} className="text-sm">
              <span className="mr-2 text-xs text-zinc-500">{at.toLocaleTimeString()}</span>
              {describe(event)}
            </li>
          ))}
        </ul>
      )}
    </Section>
  );
}

function describe(e: PipelineEvent) {
  switch (e.kind) {
    case "translated":
      return (
        <>
          <span className="text-zinc-500">{e.source}</span> → <span>{e.translated}</span>
        </>
      );
    case "skipped":
      return <span className="text-zinc-500">{e.reason}</span>;
    case "failed":
      return <span className="text-red-600">{e.message}</span>;
  }
}
