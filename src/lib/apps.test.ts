import { describe, expect, it } from "vitest";
import { formatAppList, parseAppList, withApp } from "./apps";

describe("app list", () => {
  it("splits, trims and drops blanks and duplicates", () => {
    expect(parseAppList(" Discord.exe, slack.exe ,, discord.EXE\nTelegram.exe")).toEqual([
      "Discord.exe",
      "slack.exe",
      "Telegram.exe",
    ]);
  });
  it("adds an app once", () => {
    expect(withApp(["Discord.exe"], "KakaoTalk.exe")).toEqual(["Discord.exe", "KakaoTalk.exe"]);
    expect(withApp(["Discord.exe"], "discord.exe")).toEqual(["Discord.exe"]);
  });
  it("round-trips", () => {
    expect(parseAppList(formatAppList(["Discord.exe", "slack.exe"]))).toEqual(["Discord.exe", "slack.exe"]);
  });
});
