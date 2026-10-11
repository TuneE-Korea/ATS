import { describe, expect, it } from "vitest";
import { hasCondition, move, profileFromWindow } from "./profiles";

describe("profileFromWindow", () => {
  it("names the profile after the exe", () => {
    expect(profileFromWindow("Discord.exe", "JA")).toMatchObject({ name: "Discord", process: "Discord.exe", targetLang: "JA" });
  });
});

describe("move", () => {
  it("swaps with the neighbour", () => {
    expect(move(["a", "b", "c"], 1, -1)).toEqual(["b", "a", "c"]);
  });
  it("ignores moves past the edges", () => {
    expect(move(["a", "b"], 0, -1)).toEqual(["a", "b"]);
    expect(move(["a", "b"], 1, 1)).toEqual(["a", "b"]);
  });
});

describe("hasCondition", () => {
  it("needs a process or a title", () => {
    const base = profileFromWindow("", "JA");
    expect(hasCondition(base)).toBe(false);
    expect(hasCondition({ ...base, titleContains: " reddit " })).toBe(true);
    expect(hasCondition({ ...base, domain: "5ch.net" })).toBe(true);
  });
});
