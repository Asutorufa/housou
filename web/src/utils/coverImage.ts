import type { UnifiedMetadata } from "../types";

// TMDb's extraLarge points at the original upload, which is far larger than a
// card needs. Other providers use extraLarge for a size suited to cards.
const TMDB_ORIGINAL_URL = /^https:\/\/image\.tmdb\.org\/t\/p\/original\//;

export function getCardCoverUrl(coverImage?: UnifiedMetadata["coverImage"]) {
  const { extraLarge, large } = coverImage ?? {};
  if (extraLarge && TMDB_ORIGINAL_URL.test(extraLarge) && large) return large;
  return extraLarge || large;
}
