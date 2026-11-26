export interface MorphRect {
  left: number;
  top: number;
  right: number;
  bottom: number;
  width: number;
  height: number;
}

export interface CardMorphElements {
  card: HTMLElement;
  coverWindow: HTMLElement | null;
  coverImg: HTMLImageElement | null;
}

export interface CardMorphSnapshot {
  cardRect: MorphRect;
  coverWindowRect: MorphRect | null;
  /** Full uncropped poster rect on screen (accounting for object-cover and optional hover scale). */
  coverPosterRect: MorphRect | null;
  /** Natural aspect ratio (width / height) of the poster. */
  coverAspect: number | null;
}

export interface PanelMorphFrame {
  dx: number;
  dy: number;
  scale: number;
  insetX: number;
  insetY: number;
  cardRadius: number;
  modalRadius: number;
}

export interface CoverMorphFrame {
  dx: number;
  dy: number;
  scaleX: number;
  scaleY: number;
  clipTop: number;
  clipRight: number;
  clipBottom: number;
  clipLeft: number;
}

const aspects = new Map<string, number>();
const cardRegistry = new Map<string, CardMorphElements>();

export function rememberCoverAspect(url: string, img: HTMLImageElement) {
  if (img.naturalWidth > 0 && img.naturalHeight > 0) {
    aspects.set(url, img.naturalWidth / img.naturalHeight);
  }
}

export function getCoverAspect(url: string | undefined): number | undefined {
  return url ? aspects.get(url) : undefined;
}

export function registerCardMorph(
  title: string,
  elements: CardMorphElements,
): () => void {
  cardRegistry.set(title, elements);
  return () => {
    if (cardRegistry.get(title)?.card === elements.card) {
      cardRegistry.delete(title);
    }
  };
}

function toRect(rect: DOMRect): MorphRect {
  return {
    left: rect.left,
    top: rect.top,
    right: rect.right,
    bottom: rect.bottom,
    width: rect.width,
    height: rect.height,
  };
}

function readElementScale(el: HTMLElement): number {
  if (typeof window === "undefined" || typeof DOMMatrix === "undefined") {
    return 1;
  }
  const transform = window.getComputedStyle(el).transform;
  if (!transform || transform === "none") return 1;
  try {
    const matrix = new DOMMatrix(transform);
    const scaleX = Math.hypot(matrix.a, matrix.b);
    return scaleX > 0 ? scaleX : 1;
  } catch {
    return 1;
  }
}

/**
 * Snapshot a card's live on-screen bounds and its poster's full uncropped bounds.
 * On open, `includeHoverScale` preserves any CSS `group-hover:scale-105` so the cover
 * does not snap down by 5% on the click frame. On close, we target the resting card.
 */
export function captureCardMorphSnapshot(
  title: string,
  includeHoverScale = true,
): CardMorphSnapshot | null {
  const entry = cardRegistry.get(title);
  if (!entry || !entry.card.isConnected) return null;

  const cardRect = toRect(entry.card.getBoundingClientRect());
  if (cardRect.width <= 0 || cardRect.height <= 0) return null;

  let coverWindowRect: MorphRect | null = null;
  let coverPosterRect: MorphRect | null = null;
  let coverAspect: number | null = null;

  if (entry.coverWindow && entry.coverWindow.isConnected) {
    const winRect = toRect(entry.coverWindow.getBoundingClientRect());
    if (winRect.width > 0 && winRect.height > 0) {
      coverWindowRect = winRect;
      const img = entry.coverImg;
      const aspect =
        img && img.naturalWidth > 0 && img.naturalHeight > 0
          ? img.naturalWidth / img.naturalHeight
          : img?.currentSrc
            ? getCoverAspect(img.currentSrc)
            : undefined;

      if (aspect && aspect > 0) {
        coverAspect = aspect;
        const scale =
          includeHoverScale && img && img.isConnected
            ? readElementScale(img)
            : 1;
        const winAspect = winRect.width / winRect.height;
        const baseW =
          aspect < winAspect ? winRect.width : winRect.height * aspect;
        const baseH =
          aspect < winAspect ? winRect.width / aspect : winRect.height;
        const w = baseW * scale;
        const h = baseH * scale;
        const cx = winRect.left + winRect.width / 2;
        const cy = winRect.top + winRect.height / 2;
        coverPosterRect = {
          left: cx - w / 2,
          top: cy - h / 2,
          right: cx + w / 2,
          bottom: cy + h / 2,
          width: w,
          height: h,
        };
      }
    }
  }

  return { cardRect, coverWindowRect, coverPosterRect, coverAspect };
}

/**
 * Compute the card-end panel transform + rounded `clip-path: inset(...)` parameters so the
 * modal panel visually matches `cardRect` with crisp corner radii and zero text stretching.
 */
export function computePanelMorphFrame(
  cardRect: MorphRect,
  modalRect: MorphRect,
  cardRadius = 16,
  modalRadius = 24,
): PanelMorphFrame {
  const dx =
    cardRect.left + cardRect.width / 2 - (modalRect.left + modalRect.width / 2);
  const dy =
    cardRect.top + cardRect.height / 2 - (modalRect.top + modalRect.height / 2);
  const scale = Math.max(
    1,
    cardRect.width / modalRect.width,
    cardRect.height / modalRect.height,
  );
  const insetX = Math.max(
    0,
    (modalRect.width * scale - cardRect.width) / (2 * scale),
  );
  const insetY = Math.max(
    0,
    (modalRect.height * scale - cardRect.height) / (2 * scale),
  );

  return {
    dx,
    dy,
    scale,
    insetX,
    insetY,
    cardRadius: cardRadius / scale,
    modalRadius,
  };
}

/**
 * Compute the cover `<img>` transform + clip-path parameters (relative to the transformed
 * modal panel) so its on-screen box matches `posterRect` clipped to `windowRect` at `t = 0`.
 */
export function computeCoverMorphFrame(
  posterRect: MorphRect,
  windowRect: MorphRect,
  targetCoverRect: MorphRect,
  modalRect: MorphRect,
  panelFrame: Pick<PanelMorphFrame, "dx" | "dy" | "scale">,
): CoverMorphFrame {
  const mCx = modalRect.left + modalRect.width / 2;
  const mCy = modalRect.top + modalRect.height / 2;
  const tCx = targetCoverRect.left + targetCoverRect.width / 2;
  const tCy = targetCoverRect.top + targetCoverRect.height / 2;
  const pCx = posterRect.left + posterRect.width / 2;
  const pCy = posterRect.top + posterRect.height / 2;

  const vX = tCx - mCx;
  const vY = tCy - mCy;
  const dx = (pCx - (mCx + panelFrame.dx)) / panelFrame.scale - vX;
  const dy = (pCy - (mCy + panelFrame.dy)) / panelFrame.scale - vY;
  const scaleX = posterRect.width / (targetCoverRect.width * panelFrame.scale);
  const scaleY =
    posterRect.height / (targetCoverRect.height * panelFrame.scale);

  const clipTop =
    Math.max(0, (windowRect.top - posterRect.top) / posterRect.height) * 100;
  const clipRight =
    Math.max(0, (posterRect.right - windowRect.right) / posterRect.width) * 100;
  const clipBottom =
    Math.max(0, (posterRect.bottom - windowRect.bottom) / posterRect.height) *
    100;
  const clipLeft =
    Math.max(0, (windowRect.left - posterRect.left) / posterRect.width) * 100;

  return {
    dx,
    dy,
    scaleX,
    scaleY,
    clipTop,
    clipRight,
    clipBottom,
    clipLeft,
  };
}

/**
 * Apply interpolated panel styles for `t` in `[0, 1]` (0 = card rect, 1 = modal rect).
 * Writing transform and clip-path together in one tick prevents compositor/main-thread skew.
 */
export function applyPanelMorphAt(
  el: HTMLElement,
  frame: PanelMorphFrame,
  t: number,
) {
  const inv = 1 - t;
  const dx = frame.dx * inv;
  const dy = frame.dy * inv;
  const scale = 1 + (frame.scale - 1) * inv;
  const insetX = frame.insetX * inv;
  const insetY = frame.insetY * inv;
  const radius = frame.cardRadius + (frame.modalRadius - frame.cardRadius) * t;

  el.style.transform =
    scale === 1
      ? `translate3d(${dx.toFixed(2)}px, ${dy.toFixed(2)}px, 0)`
      : `translate3d(${dx.toFixed(2)}px, ${dy.toFixed(2)}px, 0) scale(${scale.toFixed(4)})`;
  el.style.clipPath = `inset(${insetY.toFixed(2)}px ${insetX.toFixed(2)}px ${insetY.toFixed(2)}px ${insetX.toFixed(2)}px round ${radius.toFixed(2)}px)`;
}

/**
 * Apply interpolated cover styles for `t` in `[0, 1]` (0 = card cover, 1 = modal cover).
 */
export function applyCoverMorphAt(
  el: HTMLElement,
  frame: CoverMorphFrame,
  t: number,
) {
  const inv = 1 - t;
  const dx = frame.dx * inv;
  const dy = frame.dy * inv;
  const scaleX = frame.scaleX + (1 - frame.scaleX) * t;
  const scaleY = frame.scaleY + (1 - frame.scaleY) * t;
  const top = frame.clipTop * inv;
  const right = frame.clipRight * inv;
  const bottom = frame.clipBottom * inv;
  const left = frame.clipLeft * inv;

  el.style.transform = `translate3d(${dx.toFixed(2)}px, ${dy.toFixed(2)}px, 0) scale(${scaleX.toFixed(4)}, ${scaleY.toFixed(4)})`;
  el.style.clipPath = `inset(${top.toFixed(3)}% ${right.toFixed(3)}% ${bottom.toFixed(3)}% ${left.toFixed(3)}%)`;
}

export function isRectInViewport(rect: MorphRect): boolean {
  if (typeof window === "undefined") return false;
  return (
    rect.width > 0 &&
    rect.height > 0 &&
    rect.bottom > 0 &&
    rect.top < window.innerHeight &&
    rect.right > 0 &&
    rect.left < window.innerWidth
  );
}
