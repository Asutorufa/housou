import { ChevronDown } from "lucide-react";
import { motion } from "motion/react";
import { useEffect, useRef, useState } from "react";
import type { UniversalEpisode } from "../../types";

function EpisodeItem({ ep }: { ep: UniversalEpisode }) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isOverflowing, setIsOverflowing] = useState(false);
  const textRef = useRef<HTMLParagraphElement>(null);

  useEffect(() => {
    if (textRef.current) {
      setIsOverflowing(textRef.current.scrollHeight > 34);
    }
  }, [ep.overview]);

  return (
    <div
      onClick={() => {
        if (!isOverflowing) return;
        const selection = window.getSelection();
        // If the user has selected text, do not collapse/expand
        if (selection && selection.toString().length > 0) return;
        setIsExpanded(!isExpanded);
      }}
      className={`group/ep flex flex-col gap-1.5 rounded-xl border border-gray-100 bg-gray-50 p-2.5 text-sm transition-colors hover:bg-white dark:border-gray-700/50 dark:bg-gray-900/40 dark:hover:bg-gray-800 ${
        isExpanded ? "bg-white dark:bg-gray-800" : ""
      } ${isOverflowing ? "cursor-pointer" : ""}`}
    >
      <div className="flex items-center gap-3">
        <span className="w-6 shrink-0 text-center font-black text-blue-600 dark:text-blue-400">
          {ep.number}
        </span>
        <span className="truncate font-medium text-gray-700 transition-colors group-hover/ep:text-blue-600 dark:text-gray-200 dark:group-hover/ep:text-blue-400">
          {ep.title || `Episode ${ep.number}`}
        </span>
        <div className="ml-auto flex shrink-0 items-center gap-2">
          {ep.runtime && (
            <span className="self-center rounded bg-gray-100 px-1.5 py-0.5 text-[10px] font-bold text-gray-400 dark:bg-gray-800 dark:text-gray-500">
              {ep.runtime}分
            </span>
          )}
          {ep.airDate && (
            <span className="self-center text-[10px] text-gray-400 dark:text-gray-500">
              {ep.airDate}
            </span>
          )}
          {isOverflowing && (
            <ChevronDown
              size={16}
              className={`text-gray-400 transition-transform duration-300 ${
                isExpanded ? "rotate-180" : ""
              }`}
            />
          )}
        </div>
      </div>
      {ep.overview && (
        <motion.div
          initial={false}
          animate={{ height: isExpanded || !isOverflowing ? "auto" : "2rem" }}
          transition={{ duration: 0.3, ease: "easeInOut" }}
          className="overflow-hidden"
        >
          <p
            ref={textRef}
            className="pl-9 text-xs leading-4 text-gray-500 dark:text-gray-400"
          >
            {ep.overview}
          </p>
        </motion.div>
      )}
    </div>
  );
}

export default function EpisodeList({
  episodes,
}: {
  episodes?: UniversalEpisode[];
}) {
  if (!episodes || episodes.length === 0) return null;

  return (
    <div>
      <h4 className="mb-3 text-sm font-black tracking-wider text-gray-400 uppercase dark:text-gray-500">
        エピソード
      </h4>
      <div className="custom-scrollbar grid max-h-80 grid-cols-1 gap-2 overflow-y-auto pr-2">
        {episodes.map((ep: UniversalEpisode) => (
          <EpisodeItem key={ep.number} ep={ep} />
        ))}
      </div>
    </div>
  );
}
