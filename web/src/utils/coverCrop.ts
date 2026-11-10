import type { CSSProperties } from "react";

/**
 * Cards crop the cover to 3:4 while the details modal shows the whole poster. To morph between
 * the two without stretching the picture, both sides need the poster's real aspect ratio: the
 * card lays the image out at its natural shape (overflowing its 3:4 window) and the modal
 * starts from a clip that matches the card's crop.
 */
const CARD_ASPECT = 3 / 4;

const aspects = new Map<string, number>();

export function rememberCoverAspect(url: string, img: HTMLImageElement) {
  if (img.naturalWidth > 0 && img.naturalHeight > 0) {
    aspects.set(url, img.naturalWidth / img.naturalHeight);
  }
}

export function getCoverAspect(url: string | undefined): number | undefined {
  return url ? aspects.get(url) : undefined;
}

/** Box (relative to the card's 3:4 window) that equals `object-fit: cover` for this poster. */
export function cardCoverBox(aspect: number | undefined): CSSProperties {
  if (!aspect) return { inset: 0 };
  if (aspect < CARD_ASPECT) {
    const height = (CARD_ASPECT / aspect) * 100;
    return {
      left: 0,
      width: "100%",
      top: `${(100 - height) / 2}%`,
      height: `${height}%`,
    };
  }
  const width = (aspect / CARD_ASPECT) * 100;
  return {
    top: 0,
    height: "100%",
    left: `${(100 - width) / 2}%`,
    width: `${width}%`,
  };
}

/** `clip-path` that trims the full poster down to what the card shows. */
export function cardCropClipPath(aspect: number): string {
  const vertical =
    aspect < CARD_ASPECT ? ((1 - aspect / CARD_ASPECT) / 2) * 100 : 0;
  const horizontal =
    aspect > CARD_ASPECT ? ((1 - CARD_ASPECT / aspect) / 2) * 100 : 0;
  return `inset(${vertical}% ${horizontal}% ${vertical}% ${horizontal}%)`;
}
