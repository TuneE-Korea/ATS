import type { Usage } from "./types";

/** 0~100 사이 정수 퍼센트. limit이 0이면 0. */
export function usagePercent({ used, limit }: Pick<Usage, "used" | "limit">): number {
  if (limit <= 0) return 0;
  return Math.min(100, Math.round((used / limit) * 100));
}

export const USAGE_WARNING_PERCENT = 90;
