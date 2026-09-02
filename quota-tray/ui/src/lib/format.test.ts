import { describe, expect, it } from "vitest";
import { formatHeadlinePercent, meterWidthPercent } from "./format";

describe("formatHeadlinePercent", () => {
  it("does not fake zero for missing", () => {
    expect(formatHeadlinePercent(null)).toBe("—");
    expect(formatHeadlinePercent(undefined)).toBe("—");
  });

  it("shows real zero", () => {
    expect(formatHeadlinePercent(0)).toBe("0%");
  });

  it("rounds used percent", () => {
    expect(formatHeadlinePercent(62.4)).toBe("62%");
  });
});

describe("meterWidthPercent", () => {
  it("returns null when unknown so CSS can hide fill", () => {
    expect(meterWidthPercent(null)).toBeNull();
  });
});
