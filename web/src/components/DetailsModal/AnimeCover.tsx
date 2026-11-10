import { motion } from "motion/react";
import { useState } from "react";
import { MORPH_TRANSITION } from "../../styles/motion";
import type { UnifiedMetadata } from "../../types";
import { cn } from "../../utils/cn";
import { cardCropClipPath, getCoverAspect } from "../../utils/coverCrop";
import { getCardCoverUrl } from "../../utils/coverImage";

// Wider than the image so the drop shadow isn't clipped once the cover is fully revealed.
const UNCLIPPED = "inset(-20% -20% -20% -20%)";

interface AnimeCoverProps {
  info: UnifiedMetadata | null;
  title: string;
  loading?: boolean;
}

export default function AnimeCover({ info, title, loading }: AnimeCoverProps) {
  const src = info?.coverImage?.extraLarge || info?.coverImage?.large;
  const cardSrc = getCardCoverUrl(info?.coverImage);
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
        "relative flex aspect-[3/4] w-full shrink-0 items-center justify-center md:aspect-auto md:w-2/5",
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
            <img
              src={cardSrc}
              className="h-full w-full object-cover opacity-20 blur-2xl saturate-150 dark:opacity-40"
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
          className="relative z-10 max-h-full max-w-full object-contain drop-shadow-xl"
        />
      )}
      <motion.div
        exit={quickExit}
        className="absolute inset-x-0 bottom-0 z-20 h-32 bg-gradient-to-t from-white to-transparent md:hidden dark:from-gray-800"
      />
    </div>
  );
}
