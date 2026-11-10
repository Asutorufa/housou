import type { UnifiedMetadata } from "../types";

// TMDb's extraLarge points at the original upload, which is far larger than a
// card needs. Other providers use extraLarge for a size suited to cards.
const TMDB_ORIGINAL_URL = /^https:\/\/image\.tmdb\.org\/t\/p\/original\//;

// Wide enough for the details modal's cover on a 2x screen.
const TMDB_DETAIL_SIZE = "w780";

/**
 * Cover for the details modal. TMDb's original upload is typically a few megapixels: slow to
 * fetch, slow to decode and a large texture to scale every frame of the open/close animation.
 */
export function getDetailCoverUrl(coverImage?: UnifiedMetadata["coverImage"]) {
  const { extraLarge, large } = coverImage ?? {};
  if (extraLarge && TMDB_ORIGINAL_URL.test(extraLarge)) {
    return extraLarge.replace(
      TMDB_ORIGINAL_URL,
      `https://image.tmdb.org/t/p/${TMDB_DETAIL_SIZE}/`,
    );
  }
  return extraLarge || large;
}

export function getCardCoverUrl(coverImage?: UnifiedMetadata["coverImage"]) {
  const { extraLarge, large } = coverImage ?? {};
  if (extraLarge && TMDB_ORIGINAL_URL.test(extraLarge) && large) return large;
  return extraLarge || large;
}
