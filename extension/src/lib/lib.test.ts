import { describe, expect, it } from "vitest";
import { hangulRatio, shouldPreview } from "./korean";
import { placePreview } from "./placement";

describe("korean", () => {
  it("matches the Rust rule", () => {
    expect(hangulRatio("안녕!! 123 😀")).toBe(1);
    expect(hangulRatio("hello")).toBe(0);
    expect(shouldPreview("이 기능 진짜 good idea 인듯")).toBe(true);
    expect(shouldPreview("This is English")).toBe(false);
  });
  it("ignores single characters", () => {
    expect(shouldPreview("ㅋ")).toBe(false);
    expect(shouldPreview(" 네 ")).toBe(false);
    expect(shouldPreview("좋아")).toBe(true);
  });
});

describe("placePreview", () => {
  const viewport = { width: 1000, height: 800 };

  it("goes below the field when there is room", () => {
    expect(placePreview({ top: 100, bottom: 140, left: 50, width: 400 }, 80, viewport)).toEqual({ top: 146, left: 50, width: 400 });
  });

  it("flips above a field at the bottom of the screen (chat inputs)", () => {
    const p = placePreview({ top: 740, bottom: 780, left: 50, width: 400 }, 80, viewport);
    expect(p.top).toBe(740 - 6 - 80);
  });

  it("stays inside the viewport horizontally", () => {
    const p = placePreview({ top: 100, bottom: 140, left: 900, width: 300 }, 80, viewport);
    expect(p.left + p.width).toBeLessThanOrEqual(1000 - 6);
  });
});
