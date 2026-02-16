import { describe, test, expect } from "vitest";

describe("Header Options Transformation Benchmark", () => {
  const config = {
    site_meta: {
      site1: { title: "Site One", type: "tv", regions: ["US"] },
      site2: { title: "Site Two", type: "tv", regions: ["US"] },
      site3: { title: "Site Three", type: "tv", regions: ["US"] },
      site4: { title: "Site Four", type: "tv", regions: ["US"] },
      site5: { title: "Site Five", type: "tv", regions: ["US"] },
      site6: { title: "Site Six", type: "tv", regions: ["US"] },
      site7: { title: "Site Seven", type: "tv", regions: ["US"] },
      site8: { title: "Site Eight", type: "tv", regions: ["US"] },
      site9: { title: "Site Nine", type: "tv", regions: ["US"] },
      site10: { title: "Site Ten", type: "tv", regions: ["US"] },
      // ... assume more sites
      site11: { title: "Site Eleven", type: "tv", regions: ["US"] },
      site12: { title: "Site Twelve", type: "tv", regions: ["US"] },
      site13: { title: "Site Thirteen", type: "tv", regions: ["US"] },
      site14: { title: "Site Fourteen", type: "tv", regions: ["US"] },
      site15: { title: "Site Fifteen", type: "tv", regions: ["US"] },
      site16: { title: "Site Sixteen", type: "tv", regions: ["US"] },
      site17: { title: "Site Seventeen", type: "tv", regions: ["US"] },
      site18: { title: "Site Eighteen", type: "tv", regions: ["US"] },
      site19: { title: "Site Nineteen", type: "tv", regions: ["US"] },
      site20: { title: "Site Twenty", type: "tv", regions: ["US"] },
    },
  };

  test("Measures the cost of generating site options", () => {
    const iterations = 100000;
    const startTime = performance.now();

    for (let i = 0; i < iterations; i++) {
      const options = [
        { value: "all", label: "全て" },
        ...Object.entries(config.site_meta || {}).map(([key, meta]) => ({
          value: key,
          label: meta?.title || key,
        })),
      ];
      // Prevent compiler optimization
      if (options.length === 0) {
        throw new Error("Should not happen");
      }
    }

    const endTime = performance.now();
    const duration = endTime - startTime;
    console.log(
      `\n\nBenchmark Results:\nIterations: ${iterations}\nTotal Time: ${duration.toFixed(2)}ms\nAverage Time per Iteration: ${(duration / iterations).toFixed(5)}ms\n\n`,
    );

    expect(duration).toBeGreaterThan(0);
  });
});
