/** Shared by the card and the details modal so both ends of the morph move identically. */
export const MORPH_EASE = [0.32, 0.72, 0, 1] as const;
export const MORPH_TRANSITION = { duration: 0.42, ease: MORPH_EASE };
