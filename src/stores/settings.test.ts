import { describe, expect, it } from "vitest";
import { isDirty } from "./settings";
import type { Settings } from "../lib/types";

const base: Settings = {
  deeplApiKey: "",
  targetLang: "EN-US",
  hotkey: "Ctrl+Alt+T",
  mode: "hotkey",
  onlyTranslateKorean: true,
};

describe("isDirty", () => {
  it("is false for identical settings", () => {
    expect(isDirty(base, { ...base })).toBe(false);
  });
  it("detects a changed field", () => {
    expect(isDirty(base, { ...base, targetLang: "JA" })).toBe(true);
  });
  it("is false before loading", () => {
    expect(isDirty(null, base)).toBe(false);
  });
});
