import { useCallback, useEffect, useState } from "react";
import { errorMessage, ipc } from "../lib/ipc";
import { PROVIDER_LABEL, type Usage } from "../lib/types";
import { USAGE_WARNING_PERCENT, usagePercent } from "../lib/usage";
import { Section } from "./Section";

export function UsageMeter({ refreshKey }: { refreshKey: unknown }) {
  const [usages, setUsages] = useState<Usage[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setUsages(await ipc.getUsage());
      setError(null);
    } catch (e) {
      setUsages(null);
      setError(errorMessage(e));
    }
  }, []);

  // 설정 저장이나 번역이 끝날 때마다(refreshKey 변경) 다시 조회한다.
  useEffect(() => {
    void refresh();
  }, [refresh, refreshKey]);

  return (
    <Section title="이번 달 사용량">
      {usages === null && <p className="text-sm text-zinc-500">{error ?? "불러오는 중..."}</p>}
      {usages?.map((u) => {
        const percent = usagePercent(u);
        return (
          <div key={u.provider} className="space-y-1">
            <div className="flex justify-between text-sm">
              <span>{PROVIDER_LABEL[u.provider]}</span>
              <span className="text-zinc-500">
                {u.used.toLocaleString()} / {u.limit.toLocaleString()}자 ({percent}%)
              </span>
            </div>
            <div className="h-2 overflow-hidden rounded-full bg-zinc-200 dark:bg-zinc-800">
              <div
                className={`h-full ${percent >= USAGE_WARNING_PERCENT ? "bg-red-500" : "bg-indigo-500"}`}
                style={{ width: `${percent}%` }}
              />
            </div>
          </div>
        );
      })}
      <p className="text-xs text-zinc-500">Azure는 사용량 조회 API가 없어 Azure Portal에서 확인해야 합니다.</p>
    </Section>
  );
}
