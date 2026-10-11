import type { Usage } from "./types";

/** 0~100 사이 정수 퍼센트. limit이 0이면 0. */
export function usagePercent({ used, limit }: Usage): number {
  if (limit <= 0) return 0;
  return Math.min(100, Math.round((used / limit) * 100));
}

/** Phase 2에서 이 기준으로 보조 공급자 자동 전환을 붙인다. */
export const USAGE_WARNING_PERCENT = 90;
