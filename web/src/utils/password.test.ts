import { describe, expect, it } from "vitest";
import {
  PASSWORD_VALIDATION_ERRORS,
  validatePasswordComplexity,
} from "./password";

describe("validatePasswordComplexity", () => {
  it("accepts passwords that meet the policy", () => {
    expect(() => validatePasswordComplexity("Pass1234")).not.toThrow();
    expect(() => validatePasswordComplexity("Strong!Pass0")).not.toThrow();
  });

  it("rejects short passwords", () => {
    expect(() => validatePasswordComplexity("P1s")).toThrow(
      PASSWORD_VALIDATION_ERRORS.TOO_SHORT,
    );
  });

  it.each(["pass1234", "PASS1234", "Password"])(
    "rejects incomplete complexity: %s",
    (password) => {
      expect(() => validatePasswordComplexity(password)).toThrow(
        PASSWORD_VALIDATION_ERRORS.COMPLEXITY,
      );
    },
  );
});
