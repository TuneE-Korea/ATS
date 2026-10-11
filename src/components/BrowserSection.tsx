import { useEffect, useState } from "react";
import { ipc } from "../lib/ipc";
import type { ServerInfo } from "../lib/types";
import { useConfig } from "../stores/config";
import { Section, inputClass } from "./Section";

/** 브라우저 확장 연결 안내. 모드가 실시간일 때만 의미가 있지만, 토큰은 항상 보여준다. */
export function BrowserSection() {
  const token = useConfig((s) => s.saved?.secrets.pairingToken ?? "");
  const mode = useConfig((s) => s.saved?.settings.mode);
  const regenerate = useConfig((s) => s.regeneratePairingToken);
  const [info, setInfo] = useState<ServerInfo | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    // 서버는 앱 시작 직후 비동기로 뜨므로 한 번 늦게 확인한다.
    const t = setTimeout(() => void ipc.getServerInfo().then(setInfo).catch(() => setInfo(null)), 500);
    return () => clearTimeout(t);
  }, []);

  async function copy() {
    await navigator.clipboard.writeText(token);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  return (
    <Section title="브라우저 확장 (실시간 미리보기)">
      <p className="text-sm">
        상태:{" "}
        {info === null ? (
          <span className="text-zinc-500">확인 중...</span>
        ) : info.running ? (
          <span className="text-emerald-600">연결 대기 중 (127.0.0.1:{info.port})</span>
        ) : (
          <span className="text-red-600">{info.error ?? "서버가 꺼져 있습니다"}</span>
        )}
      </p>
      {mode !== "realtime" && (
        <p className="text-xs text-amber-600">
          기본 설정의 입력 방식을 "단축키 + 브라우저 실시간 미리보기"로 바꿔야 미리보기가 동작합니다.
        </p>
      )}

      <div className="flex gap-2">
        <input className={`${inputClass} font-mono`} value={token} readOnly aria-label="연결 토큰" />
        <button className="shrink-0 rounded-md border border-zinc-300 px-3 text-sm dark:border-zinc-700" onClick={() => void copy()}>
          {copied ? "복사됨" : "복사"}
        </button>
        <button
          className="shrink-0 rounded-md px-2 text-xs text-zinc-500 hover:underline"
          onClick={() => void regenerate()}
          title="토큰이 유출됐다고 생각될 때 새로 만듭니다. 확장에 다시 넣어야 합니다."
        >
          재발급
        </button>
      </div>

      <ol className="list-decimal space-y-1 pl-5 text-xs text-zinc-500">
        <li>
          프로젝트 폴더에서 <code>npm run build:ext</code>를 실행합니다.
        </li>
        <li>
          Chrome/Edge에서 <code>chrome://extensions</code> → 개발자 모드 켜기 → "압축해제된 확장 프로그램 로드" →{" "}
          <code>extension\dist</code> 폴더 선택.
        </li>
        <li>확장 아이콘 → 옵션에서 위 토큰을 붙여넣고 "연결 확인"을 누릅니다.</li>
        <li>
          입력창에 한국어를 쓰면 아래에 번역 미리보기가 뜹니다. <kbd>Tab</kbd>으로 적용, <kbd>Esc</kbd>로 닫기.
        </li>
      </ol>
    </Section>
  );
}
