import { describe, expect, it } from "vitest";
import { getSeasonLabel } from "./season";

describe("getSeasonLabel", () => {
  // Mock date: Jan 15, 2024 (Winter)
  const currentDate = new Date(2024, 0, 15);

  it("returns base label for past/current seasons in the same year", () => {
    // Winter (0) is current. Not future.
    expect(getSeasonLabel("Winter", "冬", 2024, currentDate)).toBe("冬");
  });

  it("returns base label for past years", () => {
    expect(getSeasonLabel("Winter", "冬", 2023, currentDate)).toBe("冬");
    expect(getSeasonLabel("Spring", "春", 2023, currentDate)).toBe("春");
  });

  it("returns scheduled label for future seasons in the same year", () => {
    // Current is Winter (0).
    // Spring (1) is future.
    expect(getSeasonLabel("Spring", "春", 2024, currentDate)).toBe("春 (予定)");
    expect(getSeasonLabel("Summer", "夏", 2024, currentDate)).toBe("夏 (予定)");
    expect(getSeasonLabel("Autumn", "秋", 2024, currentDate)).toBe("秋 (予定)");
  });

  it("returns scheduled label for future years", () => {
    expect(getSeasonLabel("Winter", "冬", 2025, currentDate)).toBe("冬 (予定)");
    expect(getSeasonLabel("Spring", "春", 2025, currentDate)).toBe("春 (予定)");
  });

  it("handles string years correctly", () => {
    expect(getSeasonLabel("Spring", "春", "2024", currentDate)).toBe("春 (予定)");
    expect(getSeasonLabel("Winter", "冬", "2023", currentDate)).toBe("冬");
  });

  it("handles non-numeric year strings gracefully", () => {
    expect(getSeasonLabel("Winter", "冬", "abc", currentDate)).toBe("冬");
  });

  it("handles invalid season gracefully", () => {
    expect(getSeasonLabel("Invalid", "Unknown", 2024, currentDate)).toBe(
      "Unknown",
    );
  });
});
