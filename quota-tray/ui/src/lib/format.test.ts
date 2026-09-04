import { describe, expect, it } from "vitest";
import {
  formatHeadlinePercent,
  meterWidthPercent,
  segmentedActiveCount,
  isWarningSegment,
} from "./format";

describe("format", () => {
  it("formats headline percentage cleanly", () => {
    expect(formatHeadlinePercent(null)).toBe("—");
    expect(formatHeadlinePercent(undefined)).toBe("—");
    expect(formatHeadlinePercent(75.4)).toBe("75%");
    expect(formatHeadlinePercent(0)).toBe("0%");
  });

  it("clamps meter width percent between 0 and 100", () => {
    expect(meterWidthPercent(null)).toBeNull();
    expect(meterWidthPercent(50)).toBe(50);
    expect(meterWidthPercent(-5)).toBe(0);
    expect(meterWidthPercent(105)).toBe(100);
  });

  it("calculates discrete active segment count for a 12-segment meter", () => {
    expect(segmentedActiveCount(null)).toBeNull();
    expect(segmentedActiveCount(undefined)).toBeNull();
    expect(segmentedActiveCount(0)).toBe(0);
    expect(segmentedActiveCount(25)).toBe(3); // 25% of 12 = 3
    expect(segmentedActiveCount(50)).toBe(6); // 50% of 12 = 6
    expect(segmentedActiveCount(75)).toBe(9); // 75% of 12 = 9
    expect(segmentedActiveCount(100)).toBe(12); // 100% of 12 = 12
    expect(segmentedActiveCount(105)).toBe(12); // Clamped
  });

  it("identifies warning segments for usage >= 75% (segment index >= 9 of 12)", () => {
    expect(isWarningSegment(0)).toBe(false);
    expect(isWarningSegment(8)).toBe(false);
    expect(isWarningSegment(9)).toBe(true);
    expect(isWarningSegment(11)).toBe(true);
  });
});
