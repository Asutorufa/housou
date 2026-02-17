import type { DisplayAnimeItem, UnifiedMetadata } from "../../types";

interface MultilingualTitlesProps {
  info: UnifiedMetadata | null;
  originalItem?: DisplayAnimeItem;
}

export default function MultilingualTitles({
  info,
  originalItem,
}: MultilingualTitlesProps) {
  return (
    <div className="space-y-2 rounded-2xl border border-gray-100 bg-gray-50 p-4 text-sm dark:border-gray-700/50 dark:bg-gray-900/50">
      {info?.title?.native && (
        <div className="flex gap-3">
          <span className="w-14 shrink-0 font-bold text-gray-400">日本語</span>
          <span className="text-gray-700 dark:text-gray-200">
            {info.title.native}
          </span>
        </div>
      )}
      {info?.title?.native !== info?.title?.romaji && info?.title?.romaji && (
        <div className="flex gap-3">
          <span className="w-14 shrink-0 font-bold text-gray-400">
            ローマ字
          </span>
          <span className="text-gray-700 dark:text-gray-200">
            {info.title.romaji}
          </span>
        </div>
      )}
      {/* Local data translations */}
      {originalItem?.titleTranslate &&
        Object.entries(originalItem.titleTranslate).map(([lang, titles]) => {
          if (!titles?.length) return null;
          return (
            <div key={lang} className="flex gap-3">
              <span className="w-14 shrink-0 font-bold text-gray-400 uppercase">
                {{
                  "zh-Hans": "簡体字",
                  "zh-Hant": "繁体字",
                  en: "英語",
                  ja: "日本語",
                }[lang] || lang}
              </span>
              <span className="text-gray-700 dark:text-gray-200">
                {titles.join(" / ")}
              </span>
            </div>
          );
        })}

      {info?.title?.english && (
        <div className="flex gap-3">
          <span className="w-14 shrink-0 font-bold text-gray-400">英語</span>
          <span className="text-gray-700 dark:text-gray-200">
            {info.title.english}
          </span>
        </div>
      )}
    </div>
  );
}
