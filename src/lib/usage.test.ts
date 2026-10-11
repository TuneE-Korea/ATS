import { describe, expect, it } from "vitest";
import { usagePercent } from "./usage";

describe("usagePercent", () => {
  it("rounds to an integer percent", () => {
    expect(usagePercent({ used: 123_456, limit: 500_000 })).toBe(25);
  });
  it("caps at 100", () => {
    expect(usagePercent({ used: 600_000, limit: 500_000 })).toBe(100);
  });
  it("treats a zero limit as 0%", () => {
    expect(usagePercent({ used: 10, limit: 0 })).toBe(0);
  });
});
