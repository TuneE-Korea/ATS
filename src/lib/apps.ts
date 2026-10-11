/** "Discord.exe, slack.exe" → ["Discord.exe", "slack.exe"]. 빈 칸과 중복(대소문자 무시)은 뺀다. */
export function parseAppList(input: string): string[] {
  const seen = new Set<string>();
  return input
    .split(/[,\n]/)
    .map((s) => s.trim())
    .filter((s) => s && !seen.has(s.toLowerCase()) && seen.add(s.toLowerCase()));
}

export function formatAppList(apps: string[]): string {
  return apps.join(", ");
}
