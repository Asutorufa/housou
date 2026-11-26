/** Shared easing curve for the card-to-details-modal morph. */
export const MORPH_EASE = [0.32, 0.72, 0, 1] as const;
export const MORPH_OPEN_DURATION = 0.38;
export const MORPH_CLOSE_DURATION = 0.32;
export const MORPH_TRANSITION = {
  duration: MORPH_OPEN_DURATION,
  ease: MORPH_EASE,
};
/** When the open morph is over and heavier work may start without costing it frames. */
export const MORPH_SETTLE_MS = MORPH_OPEN_DURATION * 1000 + 20;
