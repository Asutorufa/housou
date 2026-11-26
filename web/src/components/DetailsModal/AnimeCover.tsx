import { motion } from "motion/react";
import { type CSSProperties, type RefObject, useEffect, useState } from "react";
import { MORPH_SETTLE_MS } from "../../styles/motion";
import type { UnifiedMetadata } from "../../types";
import { cn } from "../../utils/cn";
import { getCoverAspect, rememberCoverAspect } from "../../utils/coverCrop";
import { getCardCoverUrl, getDetailCoverUrl } from "../../utils/coverImage";

interface AnimeCoverProps {
  info: UnifiedMetadata | null;
  title: string;
  loading?: boolean;
  fromCard?: boolean;
  initialAspect?: number | null;
  imgRef?: RefObject<HTMLImageElement | null>;
}

export default function AnimeCover({
  info,
  title,
  loading,
  fromCard,
  initialAspect,
  imgRef,
}: AnimeCoverProps) {
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

  // Keep a stable box aspect ratio so swapping cardSrc -> sharpSrc never changes the cover size.
  const [measuredAspect, setMeasuredAspect] = useState<number | null>(
    () => initialAspect ?? getCoverAspect(cardSrc) ?? null,
  );
  const aspect = measuredAspect ?? getCoverAspect(cardSrc) ?? null;

  // Morph from the card's cover only when the image is already available on mount.
  const [hadInitialSrc] = useState(() => !!src);

  const coverSizeStyle: CSSProperties | undefined = aspect
    ? {
        width: `min(100cqw, calc(100cqh * ${aspect}))`,
        height: `min(100cqh, calc(100cqw / ${aspect}))`,
      }
    : undefined;

  const quickExit = { opacity: 0, transition: { duration: 0.12 } };

  return (
    <div
      className={cn(
        // Stacked (narrow) layout: the cover takes 40dvh and collapses down to a 6rem strip as
        // the body scrolls (--cover-shrink is the body's scroll offset, set by the modal).
        // `container-type: size` lets the cover image fill the available box at its exact
        // natural aspect ratio regardless of whether cardSrc or sharpSrc is displayed.
        "relative flex h-[max(6rem,calc(40dvh-var(--cover-shrink,0px)))] w-full shrink-0 items-center justify-center [container-type:size] md:h-full md:w-2/5",
        !src && loading && "animate-pulse",
      )}
    >
      {/* Cover column backdrop fades in as the panel widens so frame 0 matches the card surface. */}
      <motion.div
        initial={{ opacity: fromCard ? 0 : 1 }}
        animate={{ opacity: 1 }}
        exit={quickExit}
        transition={{
          duration: 0.26,
          delay: fromCard ? 0.08 : 0,
          ease: "easeOut",
        }}
        className="absolute inset-0 overflow-hidden bg-gray-100 dark:bg-gray-900"
      >
        {src ? (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.32, delay: 0.1, ease: "easeOut" }}
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
          ref={imgRef}
          initial={hadInitialSrc && fromCard ? false : { opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.26, ease: "easeOut" }}
          onLoad={(event) => {
            const img = event.currentTarget;
            if (cardSrc) rememberCoverAspect(cardSrc, img);
            if (!aspect && img.naturalWidth > 0 && img.naturalHeight > 0) {
              setMeasuredAspect(img.naturalWidth / img.naturalHeight);
            }
          }}
          src={src}
          alt={title}
          style={coverSizeStyle}
          // box-shadow, not a drop-shadow filter: a filter is re-rendered for every frame of
          // the morph, and for an opaque poster the two look the same.
          className="relative z-10 max-h-full max-w-full object-contain shadow-xl"
        />
      )}
      <motion.div
        initial={{ opacity: fromCard ? 0 : 1 }}
        animate={{ opacity: 1 }}
        exit={quickExit}
        transition={{
          duration: 0.24,
          delay: fromCard ? 0.1 : 0,
          ease: "easeOut",
        }}
        className="pointer-events-none absolute inset-x-0 bottom-0 z-20 h-1/4 bg-gradient-to-t from-white to-transparent md:hidden dark:from-gray-800"
      />
    </div>
  );
}
