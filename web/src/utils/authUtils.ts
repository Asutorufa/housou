import { blake3 } from "@noble/hashes/blake3.js";
import { bytesToHex } from "@noble/hashes/utils.js";

// Frontend salt (pepper) to prevent rainbow tables on the network layer
// ideally this should be complex and unique to the deployment
const FRONTEND_SALT =
  import.meta.env.VITE_PASSWORD_SALT || "housou-frontend-default-salt";

// Helper to hash password with BLAKE3
export async function hashPassword(password: string): Promise<string> {
  const encoder = new TextEncoder();
  // Concatenate password and salt
  const data = encoder.encode(password + FRONTEND_SALT);
  const hash = blake3(data);
  return bytesToHex(hash);
}

// Password complexity validation
export function validatePasswordComplexity(password: string): void {
  if (password.length < 8) {
    throw new Error("パスワードは8文字以上である必要があります");
  }

  const hasUppercase = /[A-Z]/.test(password);
  const hasLowercase = /[a-z]/.test(password);
  const hasDigit = /[0-9]/.test(password);

  if (!hasUppercase || !hasLowercase || !hasDigit) {
    throw new Error(
      "パスワードには、大文字、小文字、数字をそれぞれ1文字以上含める必要があります",
    );
  }
}
