import { describe, expect, it } from "vitest";
import { formatTime, progressValue } from "../src/format";

describe("timer formatting", () => {
  it("formats long sessions without losing hours", () => {
    expect(formatTime(7_200)).toBe("120:00");
    expect(formatTime(65)).toBe("1:05");
  });

  it("clamps progress to a safe range", () => {
    expect(progressValue(20, 20)).toBe(1);
    expect(progressValue(-2, 20)).toBe(0);
    expect(progressValue(2, 0)).toBe(0);
  });
});
