import { useCallback, useEffect, useState } from "react";
import { errorMessage, ipc } from "../lib/ipc";
import type { Usage } from "../lib/types";
import { USAGE_WARNING_PERCENT, usagePercent } from "../lib/usage";
import { Section } from "./Section";

export function UsageMeter({ refreshKey }: { refreshKey: unknown }) {
  const [usage, setUsage] = useState<Usage | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setUsage(await ipc.getUsage());
      setError(null);
    } catch (e) {
      setUsage(null);
      setError(errorMessage(e));
    }
  }, []);

  // 설정 저장이나 번역이 끝날 때마다(refreshKey 변경) 다시 조회한다.
  useEffect(() => {
    void refresh();
  }, [refresh, refreshKey]);

  const percent = usage ? usagePercent(usage) : 0;
  return (
    <Section title="이번 달 DeepL 사용량">
      {usage ? (
        <>
          <div className="h-2 overflow-hidden rounded-full bg-zinc-200 dark:bg-zinc-800">
            <div
              className={`h-full ${percent >= USAGE_WARNING_PERCENT ? "bg-red-500" : "bg-blue-500"}`}
              style={{ width: `${percent}%` }}
            />
          </div>
          <p className="text-sm text-zinc-500">
            {usage.used.toLocaleString()} / {usage.limit.toLocaleString()}자 ({percent}%)
          </p>
        </>
      ) : (
        <p className="text-sm text-zinc-500">{error ?? "불러오는 중..."}</p>
      )}
    </Section>
  );
}
