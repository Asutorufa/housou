import { Clock, PlayCircle, Star } from "lucide-react";
import type { UnifiedMetadata } from "../../types";

export default function InfoBadges({ info }: { info: UnifiedMetadata | null }) {
  return (
    <div className="flex flex-wrap gap-2">
      {!!info?.averageScore && info.averageScore > 0 && (
        <div className="flex items-center gap-1.5 rounded-full border border-yellow-200/50 bg-yellow-50 px-3 py-1 text-sm font-bold text-yellow-700 dark:border-yellow-700/30 dark:bg-yellow-900/20 dark:text-yellow-400">
          <Star size={14} className="fill-current" />
          {info.averageScore}%
        </div>
      )}
      {info?.episodes && (
        <div className="flex items-center gap-1.5 rounded-full border border-purple-200/50 bg-purple-50 px-3 py-1 text-sm font-bold text-purple-700 dark:border-purple-700/30 dark:bg-purple-900/20 dark:text-purple-400">
          <PlayCircle size={14} />
          {info.episodes}話
        </div>
      )}
      {info?.totalSeasons && (
        <div className="flex items-center gap-1 rounded-full bg-purple-100 px-3 py-1 text-sm font-bold text-purple-700 dark:bg-purple-900/30 dark:text-purple-300">
          シーズン{info.currentSeason || 1} / 全{info.totalSeasons}シーズン
        </div>
      )}
      {!!info?.runtime && info.runtime > 0 && (
        <div className="flex items-center gap-1.5 rounded-full border border-gray-200/50 bg-gray-100 px-3 py-1 text-sm font-bold text-gray-700 dark:border-gray-700/30 dark:bg-gray-700/50 dark:text-gray-300">
          <Clock size={14} className="stroke-current" />
          {info.runtime}分
        </div>
      )}
      {info?.contentRating && (
        <div className="flex items-center gap-1.5 rounded-full border border-gray-200/50 bg-gray-100 px-3 py-1 text-sm font-bold text-gray-700 dark:border-gray-700/30 dark:bg-gray-700/50 dark:text-gray-300">
          {info.contentRating}
        </div>
      )}
      {info?.genres?.slice(0, 3).map((g: string) => (
        <span
          key={g}
          className="rounded-full bg-gray-100 px-3 py-1 text-sm font-medium text-gray-600 dark:bg-gray-700/50 dark:text-gray-300"
        >
          {g}
        </span>
      ))}
    </div>
  );
}
