import { motion } from "motion/react";
import { useEffect, useState } from "react";
import { MORPH_SETTLE_MS, MORPH_TRANSITION } from "../../styles/motion";
import type { UnifiedMetadata } from "../../types";
import { cn } from "../../utils/cn";
import { cardCropClipPath, getCoverAspect } from "../../utils/coverCrop";
import { getCardCoverUrl, getDetailCoverUrl } from "../../utils/coverImage";

// Wider than the image so its shadow isn't clipped once the cover is fully revealed.
const UNCLIPPED = "inset(-20% -20% -20% -20%)";

interface AnimeCoverProps {
  info: UnifiedMetadata | null;
  title: string;
  loading?: boolean;
}

export default function AnimeCover({ info, title, loading }: AnimeCoverProps) {
  const detailSrc = getDetailCoverUrl(info?.coverImage);
  const cardSrc = getCardCoverUrl(info?.coverImage);
  // The morph runs on the card's rendition: it is already downloaded and decoded, so the cover
  // is there from the first frame. The sharper one is swapped in once it is decoded and the
  // morph is over, keeping its download, decode and texture upload out of the animation.
  const [sharpSrc, setSharpSrc] = useState<string>();
  useEffect(() => {
    if (!detailSrc || detailSrc === cardSrc) return;
    let cancelled = false;
    const img = new Image();
    img.src = detailSrc;
    const settled = new Promise((resolve) =>
      setTimeout(resolve, MORPH_SETTLE_MS),
    );
    Promise.all([img.decode(), settled]).then(
      () => !cancelled && setSharpSrc(detailSrc),
      () => {}, // Keep the card's rendition if the sharper one fails to load.
    );
    return () => {
      cancelled = true;
    };
  }, [detailSrc, cardSrc]);
  const src = (sharpSrc === detailSrc && sharpSrc) || cardSrc || detailSrc;
  // Morph from the card's cover only when the image is there as the modal opens. A cover that
  // arrives later (deep link, slow metadata) fades in instead of flying out of the grid.
  const [morphFromCard] = useState(() => !!src);
  // The card shows a 3:4 crop of the poster; start from (and return to) that same crop.
  const [cardCrop] = useState(() => {
    // The card may load a smaller rendition of the same poster; it is the one that was measured.
    const aspect = getCoverAspect(cardSrc);
    return aspect ? cardCropClipPath(aspect) : null;
  });

  const quickExit = { opacity: 0, transition: { duration: 0.12 } };

  // Nothing here may clip the cover: while closing it flies back to the card, outside this
  // column. The surface and the blurred backdrop are clipped on their own layer instead.
  return (
    <div
      className={cn(
        // Stacked (narrow) layout: the cover is capped so the body below always keeps most of
        // the panel. Uncapped, a 3:4 box as wide as the panel can be taller than the panel.
        "relative flex aspect-[3/4] max-h-[40dvh] w-full shrink-0 items-center justify-center md:aspect-auto md:max-h-none md:w-2/5",
        !src && loading && "animate-pulse",
      )}
    >
      <motion.div
        exit={quickExit}
        className="absolute inset-0 overflow-hidden bg-gray-100 dark:bg-gray-900"
      >
        {src ? (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.4, delay: 0.1 }}
            className="h-full w-full"
            aria-hidden="true"
          >
            {/* Blurred at a quarter size and scaled back up: same look, 1/16 of the pixels. */}
            <img
              src={cardSrc || src}
              className="h-1/4 w-1/4 origin-top-left scale-[4] object-cover opacity-20 blur-[10px] saturate-150 dark:opacity-40"
              alt=""
            />
          </motion.div>
        ) : (
          !loading && (
            <div className="flex h-full w-full items-center justify-center text-gray-400 italic">
              No image available
            </div>
          )
        )}
      </motion.div>
      {src && (
        <motion.img
          layoutId={morphFromCard ? `image-${title}` : undefined}
          layoutCrossfade={false}
          initial={
            morphFromCard
              ? cardCrop
                ? { clipPath: cardCrop }
                : false
              : { opacity: 0 }
          }
          animate={
            cardCrop ? { opacity: 1, clipPath: UNCLIPPED } : { opacity: 1 }
          }
          exit={cardCrop ? { clipPath: cardCrop } : undefined}
          transition={{ ...MORPH_TRANSITION, opacity: { duration: 0.3 } }}
          src={src}
          alt={title}
          // box-shadow, not a drop-shadow filter: a filter is re-rendered for every frame of
          // the morph, and for an opaque poster the two look the same.
          className="relative z-10 max-h-full max-w-full object-contain shadow-xl"
        />
      )}
      <motion.div
        exit={quickExit}
        className="absolute inset-x-0 bottom-0 z-20 h-20 bg-gradient-to-t from-white to-transparent md:hidden dark:from-gray-800"
      />
    </div>
  );
}
