import { motion } from "motion/react";
import type { UnifiedMetadata } from "../../types";

interface AnimeCoverProps {
  info: UnifiedMetadata | null;
  title: string;
}

export default function AnimeCover({ info, title }: AnimeCoverProps) {
  return (
    <motion.div
      layoutId={`image-${title}`}
      className="relative flex aspect-[3/4] w-full items-center justify-center overflow-hidden bg-gray-100 md:aspect-auto md:w-2/5 dark:bg-gray-900"
    >
      {info?.coverImage?.extraLarge || info?.coverImage?.large ? (
        <>
          <img
            src={info.coverImage.extraLarge || info.coverImage.large}
            className="absolute inset-0 h-full w-full object-cover opacity-20 blur-2xl saturate-150 dark:opacity-40"
            aria-hidden="true"
            alt=""
          />
          <img
            src={info.coverImage.extraLarge || info.coverImage.large}
            alt={title}
            className="relative z-10 max-h-full max-w-full object-contain drop-shadow-xl"
          />
        </>
      ) : (
        <div className="flex h-full w-full items-center justify-center text-gray-400 italic">
          No image available
        </div>
      )}
      <div className="absolute inset-x-0 bottom-0 z-20 h-32 bg-gradient-to-t from-white to-transparent md:hidden dark:from-gray-800" />
    </motion.div>
  );
}
