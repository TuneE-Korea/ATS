import { useEffect, useState } from "react";
import { ActivityLog, type LoggedEvent } from "./components/ActivityLog";
import { GeneralSection } from "./components/GeneralSection";
import { ProfilesSection } from "./components/ProfilesSection";
import { ProvidersSection } from "./components/ProvidersSection";
import { SaveBar } from "./components/SaveBar";
import { TranslateTester } from "./components/TranslateTester";
import { UsageMeter } from "./components/UsageMeter";
import { onPipelineEvent } from "./lib/ipc";
import { useConfig } from "./stores/config";

const MAX_LOG = 20;

export default function App() {
  const load = useConfig((s) => s.load);
  const saved = useConfig((s) => s.saved);
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
    <main className="min-h-screen space-y-4 bg-zinc-50 px-4 pt-4 text-zinc-900 dark:bg-zinc-950 dark:text-zinc-100">
      <header className="flex items-center gap-3">
        <img src="/logo.svg" alt="" className="h-9 w-9" />
        <div>
          <h1 className="text-lg font-bold">ATS</h1>
          <p className="text-sm text-zinc-500">
            입력창에서 <kbd className="rounded border px-1">{saved?.settings.hotkey ?? "..."}</kbd>를 누르면 내용
            전체가 번역됩니다. 창을 닫아도 트레이에서 계속 동작합니다.
          </p>
        </div>
      </header>
      <GeneralSection />
      <ProvidersSection />
      <ProfilesSection />
      <UsageMeter refreshKey={`${JSON.stringify(saved?.secrets)}-${log.length}`} />
      <TranslateTester />
      <ActivityLog items={log} />
      <SaveBar />
    </main>
  );
}
