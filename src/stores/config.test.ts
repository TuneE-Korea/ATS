import { beforeEach, describe, expect, it } from "vitest";
import { isDirty, useConfig } from "./config";
import type { Config } from "../lib/types";

const base: Config = {
  settings: {
    targetLang: "EN-US",
    hotkey: "Ctrl+Alt+T",
    mode: "hotkey",
    onlyTranslateKorean: true,
    realtimeDebounceMs: 600,
    desktopRealtime: false,
    desktopRealtimeApps: ["Discord.exe", "slack.exe"],
    providerOrder: ["deepl", "azure"],
    profiles: [],
  },
  secrets: { deeplApiKey: "", azureKey: "", azureRegion: "", pairingToken: "t" },
};

describe("isDirty", () => {
  it("is false for identical config", () => {
    expect(isDirty(base, structuredClone(base))).toBe(false);
  });
  it("detects a nested change", () => {
    expect(isDirty(base, { ...base, secrets: { ...base.secrets, azureKey: "x" } })).toBe(true);
  });
  it("is false before loading", () => {
    expect(isDirty(null, base)).toBe(false);
  });
});

describe("profile editing", () => {
  beforeEach(() => useConfig.setState({ saved: base, draft: structuredClone(base) }));

  it("adds, updates, moves and removes profiles without touching saved", () => {
    const s = useConfig.getState();
    s.addProfile({ name: "A", process: "a.exe", titleContains: "", domain: "", targetLang: "JA", enabled: true });
    s.addProfile({ name: "B", process: "b.exe", titleContains: "", domain: "", targetLang: "DE", enabled: true });
    s.updateProfile(0, { targetLang: "FR" });
    s.moveProfile(1, -1);

    const names = () => useConfig.getState().draft!.settings.profiles.map((p) => `${p.name}:${p.targetLang}`);
    expect(names()).toEqual(["B:DE", "A:FR"]);

    useConfig.getState().removeProfile(0);
    expect(names()).toEqual(["A:FR"]);
    expect(useConfig.getState().saved!.settings.profiles).toEqual([]);
  });
});

describe("enableDesktopPreview", () => {
  beforeEach(() => useConfig.setState({ saved: base, draft: structuredClone(base) }));

  it("turns on realtime + desktop preview and adds the app once", () => {
    useConfig.getState().enableDesktopPreview("KakaoTalk.exe");
    useConfig.getState().enableDesktopPreview("kakaotalk.exe");
    const s = useConfig.getState().draft!.settings;
    expect(s.mode).toBe("realtime");
    expect(s.desktopRealtime).toBe(true);
    expect(s.desktopRealtimeApps).toEqual(["Discord.exe", "slack.exe", "KakaoTalk.exe"]);
    expect(useConfig.getState().saved!.settings.desktopRealtime).toBe(false);
  });
});
