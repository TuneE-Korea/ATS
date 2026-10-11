import { beforeEach, describe, expect, it } from "vitest";
import { isDirty, useConfig } from "./config";
import type { Config } from "../lib/types";

const base: Config = {
  settings: {
    targetLang: "EN-US",
    hotkey: "Ctrl+Alt+T",
    mode: "hotkey",
    onlyTranslateKorean: true,
    providerOrder: ["deepl", "azure"],
    profiles: [],
  },
  secrets: { deeplApiKey: "", azureKey: "", azureRegion: "" },
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
    s.addProfile({ name: "A", process: "a.exe", titleContains: "", targetLang: "JA", enabled: true });
    s.addProfile({ name: "B", process: "b.exe", titleContains: "", targetLang: "DE", enabled: true });
    s.updateProfile(0, { targetLang: "FR" });
    s.moveProfile(1, -1);

    const names = () => useConfig.getState().draft!.settings.profiles.map((p) => `${p.name}:${p.targetLang}`);
    expect(names()).toEqual(["B:DE", "A:FR"]);

    useConfig.getState().removeProfile(0);
    expect(names()).toEqual(["A:FR"]);
    expect(useConfig.getState().saved!.settings.profiles).toEqual([]);
  });
});
