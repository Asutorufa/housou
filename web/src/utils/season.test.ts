import { describe, expect, it } from "vitest";
import { getSeasonOptions } from "./season";

describe("getSeasonOptions", () => {
  it("marks future seasons correctly in current year", () => {
    // Current date: Feb 2024 (Month 2)
    // Winter (1) < 2 -> Current
    // Spring (4) > 2 -> Future
    const options = getSeasonOptions("2024", 2024, 2);

    expect(options.find((o) => o.value === "Winter")?.label).toBe("冬");
    expect(options.find((o) => o.value === "Spring")?.label).toBe("春 (予定)");
    expect(options.find((o) => o.value === "Summer")?.label).toBe("夏 (予定)");
    expect(options.find((o) => o.value === "Autumn")?.label).toBe("秋 (予定)");
  });

  it("marks all seasons as future in future year", () => {
    // Current date: Feb 2024
    // Selected year: 2025
    const options = getSeasonOptions("2025", 2024, 2);

    expect(options.find((o) => o.value === "Winter")?.label).toBe("冬 (予定)");
    expect(options.find((o) => o.value === "Spring")?.label).toBe("春 (予定)");
    expect(options.find((o) => o.value === "Summer")?.label).toBe("夏 (予定)");
    expect(options.find((o) => o.value === "Autumn")?.label).toBe("秋 (予定)");
  });

  it("marks no seasons as future in past year", () => {
    // Current date: Feb 2024
    // Selected year: 2023
    const options = getSeasonOptions("2023", 2024, 2);

    expect(options.find((o) => o.value === "Winter")?.label).toBe("冬");
    expect(options.find((o) => o.value === "Spring")?.label).toBe("春");
    expect(options.find((o) => o.value === "Summer")?.label).toBe("夏");
    expect(options.find((o) => o.value === "Autumn")?.label).toBe("秋");
  });

  it("handles invalid year string gracefully", () => {
    const options = getSeasonOptions("invalid", 2024, 2);
    expect(options.find((o) => o.value === "Winter")?.label).toBe("冬");
    expect(options.find((o) => o.value === "Spring")?.label).toBe("春");
    expect(options.find((o) => o.value === "Summer")?.label).toBe("夏");
    expect(options.find((o) => o.value === "Autumn")?.label).toBe("秋");
  });
});
