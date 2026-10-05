export const PASSWORD_VALIDATION_ERRORS = {
  TOO_SHORT: "パスワードは8文字以上である必要があります",
  COMPLEXITY:
    "パスワードには、大文字、小文字、数字をそれぞれ1文字以上含める必要があります",
} as const;

export function validatePasswordComplexity(password: string): void {
  if (Array.from(password).length < 8) {
    throw new Error(PASSWORD_VALIDATION_ERRORS.TOO_SHORT);
  }

  const hasUppercase = /[A-Z]/.test(password);
  const hasLowercase = /[a-z]/.test(password);
  const hasDigit = /[0-9]/.test(password);

  if (!hasUppercase || !hasLowercase || !hasDigit) {
    throw new Error(PASSWORD_VALIDATION_ERRORS.COMPLEXITY);
  }
}
