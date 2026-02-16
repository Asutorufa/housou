import { describe, it, expect } from "vitest";
import { isValidUrl } from "./urlUtils";

describe("isValidUrl", () => {
  it("should return true for valid http URLs", () => {
    expect(isValidUrl("http://example.com")).toBe(true);
    expect(isValidUrl("http://www.example.com/foo/bar")).toBe(true);
  });

  it("should return true for valid https URLs", () => {
    expect(isValidUrl("https://example.com")).toBe(true);
    expect(isValidUrl("https://sub.example.com?q=1")).toBe(true);
  });

  it("should return false for javascript: URLs", () => {
    expect(isValidUrl("javascript:alert(1)")).toBe(false);
    expect(isValidUrl("javascript:void(0)")).toBe(false);
  });

  it("should return false for other protocols", () => {
    expect(isValidUrl("ftp://example.com")).toBe(false);
    expect(isValidUrl("file:///etc/passwd")).toBe(false);
    expect(isValidUrl("mailto:user@example.com")).toBe(false);
  });

  it("should return false for invalid URLs", () => {
    expect(isValidUrl("not a url")).toBe(false);
    expect(isValidUrl("/relative/path")).toBe(false);
    expect(isValidUrl("//example.com")).toBe(false);
    expect(isValidUrl("")).toBe(false);
    expect(isValidUrl(null)).toBe(false);
    expect(isValidUrl(undefined)).toBe(false);
  });
});
