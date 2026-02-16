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
