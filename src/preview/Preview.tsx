// 데스크톱 앱 위에 뜨는 미리보기 카드. 내용은 Rust(desktop_preview.rs)가 이벤트로 보내 준다.
// 이 창은 포커스를 받지 않으므로 키 입력(Tab·Esc)은 Rust가 전역 단축키로 처리한다.

import { useLayoutEffect, useRef, useState } from "react";
import { ipc, onPreviewEvent } from "../lib/ipc";
import { PROVIDER_LABEL, type PreviewPayload } from "../lib/types";

export function Preview() {
  const [payload, setPayload] = useState<PreviewPayload>({ kind: "loading" });
  const card = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    const unlisten = onPreviewEvent(setPayload);
    return () => void unlisten.then((f) => f());
  }, []);

  // 내용 높이가 바뀔 때마다 Rust에 알려서 창 크기와 위치(입력창 위/아래)를 다시 잡게 한다.
  useLayoutEffect(() => {
    if (card.current) void ipc.previewLayout(Math.ceil(card.current.getBoundingClientRect().height) + 2);
  }, [payload]);

  return (
    <div
      ref={card}
      className="m-px rounded-xl border border-zinc-200 bg-white px-3 py-2 text-sm text-zinc-900 shadow-lg dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100"
    >
      {payload.kind === "loading" && <p className="text-zinc-500">번역 중…</p>}
      {payload.kind === "error" && <p className="text-red-600">{payload.message}</p>}
      {payload.kind === "result" && (
        <>
          <p className="whitespace-pre-wrap break-words">{payload.text}</p>
          <p className="mt-1 flex gap-2 text-[11px] text-zinc-500">
            <span>
              <b className="text-indigo-600 dark:text-indigo-300">Tab</b> 적용 ·{" "}
              <b className="text-indigo-600 dark:text-indigo-300">Esc</b> 닫기
            </span>
            <span className="ml-auto">
              {PROVIDER_LABEL[payload.provider]}
              {payload.profile ? ` · ${payload.profile}` : ""}
            </span>
          </p>
        </>
      )}
    </div>
  );
}
