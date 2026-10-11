/** "Discord.exe, slack.exe" → ["Discord.exe", "slack.exe"]. 빈 칸과 중복(대소문자 무시)은 뺀다. */
export function parseAppList(input: string): string[] {
  const seen = new Set<string>();
  return input
    .split(/[,\n]/)
    .map((s) => s.trim())
    .filter((s) => s && !seen.has(s.toLowerCase()) && seen.add(s.toLowerCase()));
}

/** 목록에 없으면(대소문자 무시) 끝에 붙인다. */
export function withApp(apps: string[], app: string): string[] {
  return apps.some((a) => a.toLowerCase() === app.toLowerCase()) ? apps : [...apps, app];
}

export function formatAppList(apps: string[]): string {
  return apps.join(", ");
}
