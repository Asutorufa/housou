import { blake3 } from "@noble/hashes/blake3.js";
import { bytesToHex } from "@noble/hashes/utils.js";

// Legacy frontend pre-hash retained only to authenticate accounts created by
// older clients. New passwords are sent over HTTPS and hashed with Argon2 server-side.
const FRONTEND_SALT =
  import.meta.env.VITE_PASSWORD_SALT || "housou-frontend-default-salt";

export async function hashPassword(password: string): Promise<string> {
  const encoder = new TextEncoder();
  // Concatenate password and salt
  const data = encoder.encode(password + FRONTEND_SALT);
  const hash = blake3(data);
  return bytesToHex(hash);
}
