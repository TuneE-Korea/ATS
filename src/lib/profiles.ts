import type { Profile } from "./types";

/** 활성 창 정보로 새 프로필 초안을 만든다. 로그의 "이 앱으로 프로필 만들기"에서 쓴다. */
export function profileFromWindow(process: string, targetLang: string): Profile {
  return {
    name: process.replace(/\.exe$/i, "") || "새 프로필",
    process,
    titleContains: "",
    targetLang,
    enabled: true,
  };
}

/** 배열에서 i번째 항목을 위(-1) 또는 아래(+1)로 옮긴 새 배열. 범위를 벗어나면 그대로. */
export function move<T>(items: readonly T[], i: number, dir: -1 | 1): T[] {
  const j = i + dir;
  if (j < 0 || j >= items.length) return [...items];
  const next = [...items];
  [next[i], next[j]] = [next[j], next[i]];
  return next;
}

/** 조건이 하나도 없으면 Rust 쪽에서 절대 일치하지 않는다. 화면에서 미리 경고한다. */
export function hasCondition(p: Profile): boolean {
  return p.process.trim() !== "" || p.titleContains.trim() !== "";
}
