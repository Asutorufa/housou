import { describe, it, expect } from "vitest";
import { validatePasswordComplexity } from "./authUtils";

describe("validatePasswordComplexity", () => {
  it("should accept a valid password", () => {
    expect(() => validatePasswordComplexity("Pass1234")).not.toThrow();
    expect(() => validatePasswordComplexity("Strong!Pass0")).not.toThrow();
  });

  it("should throw if password is too short", () => {
    expect(() => validatePasswordComplexity("P1s")).toThrow(
      "パスワードは8文字以上である必要があります",
    );
  });

  it("should throw if missing uppercase", () => {
    expect(() => validatePasswordComplexity("pass1234")).toThrow(
      "パスワードには、大文字、小文字、数字をそれぞれ1文字以上含める必要があります",
    );
  });

  it("should throw if missing lowercase", () => {
    expect(() => validatePasswordComplexity("PASS1234")).toThrow(
      "パスワードには、大文字、小文字、数字をそれぞれ1文字以上含める必要があります",
    );
  });

  it("should throw if missing digit", () => {
    expect(() => validatePasswordComplexity("Password")).toThrow(
      "パスワードには、大文字、小文字、数字をそれぞれ1文字以上含める必要があります",
    );
  });
});
