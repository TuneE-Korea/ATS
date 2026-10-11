import { useEffect, useState } from "react";
import { ActivityLog, type LoggedEvent } from "./components/ActivityLog";
import { SettingsForm } from "./components/SettingsForm";
import { TranslateTester } from "./components/TranslateTester";
import { UsageMeter } from "./components/UsageMeter";
import { onPipelineEvent } from "./lib/ipc";
import { useSettings } from "./stores/settings";

const MAX_LOG = 20;

export default function App() {
  const load = useSettings((s) => s.load);
  const saved = useSettings((s) => s.saved);
  const [log, setLog] = useState<LoggedEvent[]>([]);

  useEffect(() => {
    void load();
  }, [load]);

  // 단축키 파이프라인(Rust)이 끝날 때마다 보내는 이벤트를 받는다.
  useEffect(() => {
    const unlisten = onPipelineEvent((event) =>
      setLog((prev) => [{ at: new Date(), event }, ...prev].slice(0, MAX_LOG)),
    );
    return () => {
      void unlisten.then((off) => off());
    };
  }, []);

  return (
    <main className="min-h-screen space-y-4 bg-zinc-50 p-4 text-zinc-900 dark:bg-zinc-950 dark:text-zinc-100">
      <header>
        <h1 className="text-lg font-bold">ATS</h1>
        <p className="text-sm text-zinc-500">
          입력창에서 <kbd className="rounded border px-1">{saved?.hotkey ?? "..."}</kbd>를 누르면 내용 전체가
          번역됩니다. 창을 닫아도 트레이에서 계속 동작합니다.
        </p>
      </header>
      <SettingsForm />
      <UsageMeter refreshKey={`${saved?.deeplApiKey}-${log.length}`} />
      <TranslateTester />
      <ActivityLog items={log} />
    </main>
  );
}
