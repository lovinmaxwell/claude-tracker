import { describe, expect, it } from "vitest";
import { clampPollInterval } from "./settings_validate";

describe("clampPollInterval", () => {
  it("clamps to 60..120", () => {
    expect(clampPollInterval(10)).toBe(60);
    expect(clampPollInterval(60)).toBe(60);
    expect(clampPollInterval(90)).toBe(90);
    expect(clampPollInterval(120)).toBe(120);
    expect(clampPollInterval(500)).toBe(120);
  });

  it("falls back for non-finite", () => {
    expect(clampPollInterval(Number.NaN)).toBe(60);
    expect(clampPollInterval(Number.POSITIVE_INFINITY)).toBe(60);
  });
});
