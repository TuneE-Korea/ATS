import { describe, expect, it } from "vitest";
import { formatAppList, parseAppList } from "./apps";

describe("app list", () => {
  it("splits, trims and drops blanks and duplicates", () => {
    expect(parseAppList(" Discord.exe, slack.exe ,, discord.EXE\nTelegram.exe")).toEqual([
      "Discord.exe",
      "slack.exe",
      "Telegram.exe",
    ]);
  });
  it("round-trips", () => {
    expect(parseAppList(formatAppList(["Discord.exe", "slack.exe"]))).toEqual(["Discord.exe", "slack.exe"]);
  });
});
