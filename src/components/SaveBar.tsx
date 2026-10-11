import { isDirty, useConfig } from "../stores/config";

/** 화면 아래에 붙어 있는 저장 막대. 변경 사항이 있을 때만 버튼이 활성화된다. */
export function SaveBar() {
  const { saved, draft, saving, error, reset, save } = useConfig();
  const dirty = isDirty(saved, draft);

  return (
    <div className="sticky bottom-0 -mx-4 flex items-center gap-2 border-t border-zinc-200 bg-zinc-50/95 px-4 py-3 backdrop-blur dark:border-zinc-800 dark:bg-zinc-950/95">
      <p className={`flex-1 text-sm ${error ? "text-red-600" : "text-zinc-500"}`}>
        {error ?? (dirty ? "저장하지 않은 변경 사항이 있습니다" : "모든 변경 사항이 저장되었습니다")}
      </p>
      <button
        className="rounded-md px-3 py-2 text-sm text-zinc-600 disabled:opacity-40"
        onClick={reset}
        disabled={!dirty || saving}
      >
        되돌리기
      </button>
      <button
        className="rounded-md bg-indigo-600 px-4 py-2 text-sm font-medium text-white disabled:opacity-40"
        onClick={() => void save()}
        disabled={!dirty || saving}
      >
        {saving ? "저장 중..." : "저장"}
      </button>
    </div>
  );
}
