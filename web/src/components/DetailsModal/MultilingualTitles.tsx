import type { DisplayAnimeItem, UnifiedMetadata } from "../../types";

interface MultilingualTitlesProps {
  info: UnifiedMetadata | null;
  originalItem?: DisplayAnimeItem;
}

export default function MultilingualTitles({
  info,
  originalItem,
}: MultilingualTitlesProps) {
  // Helper to normalize strings for comparison
  const normalizeTitle = (title: string | undefined | null) => {
    if (!title) return "";
    return title.toLowerCase().replace(/[\s:;,/|-]/g, "");
  };

  // Global set to keep track of seen titles for deduplication (case-insensitive, no spaces)
  const seenTitles = new Set<string>();

  // Helper check to see if a string is substantially similar to an already seen string
  const hasSeenTitle = (normalized: string) => {
    if (seenTitles.has(normalized)) return true;

    // Check if the new title is a substring of an existing title or vice versa
    if (normalized.length > 5) {
      for (const seen of seenTitles) {
        if (seen.includes(normalized) || normalized.includes(seen)) {
          return true;
        }
      }
    }
    return false;
  };

  // Group everything by display label
  const displayGroups = new Map<string, string[]>();

  const addTitleToGroup = (label: string, title?: string | null) => {
    if (!title) return;
    // split if there are multiple titles joined by / or |
    const splitTitles = title.split(/\s*(?:\/|\|)\s*/).filter(Boolean);

    if (!displayGroups.has(label)) {
      displayGroups.set(label, []);
    }
    const group = displayGroups.get(label)!;

    for (const t of splitTitles) {
      const normalized = normalizeTitle(t);
      if (!hasSeenTitle(normalized)) {
        group.push(t);
        seenTitles.add(normalized);
      }
    }
  };

  // 1. Add base titles
  addTitleToGroup("日本語", info?.title?.native);
  if (info?.title?.native !== info?.title?.romaji) {
    addTitleToGroup("ローマ字", info?.title?.romaji);
  }

  // 2. Add translations
  const addTranslations = (
    source: Record<string, string[] | undefined> | null | undefined,
  ) => {
    if (!source) return;
    Object.entries(source).forEach(([lang, titles]) => {
      if (!titles?.length) return;

      let label = lang;
      // map language to label
      switch (lang) {
        case "zh-Hans":
        case "CN":
          label = "簡体字";
          break;
        case "zh-Hant":
        case "TW":
        case "HK":
          label = "繁体字";
          break;
        case "ja":
        case "JP":
          label = "日本語";
          break;
        case "en":
        case "US":
        case "GB":
          label = "英語";
          break;
        default:
          try {
            // Attempt to translate the region code into a Japanese country name natively
            const regionName = new Intl.DisplayNames(["ja"], {
              type: "region",
            }).of(lang);
            if (regionName && regionName !== lang) {
              label = `${regionName}語`;
            }
          } catch {
            // Ignore if 'lang' is not a valid region code and fallback to the raw lang string
          }
          break;
      }

      titles.forEach((t) => addTitleToGroup(label, t));
    });
  };

  addTranslations(info?.titleTranslate);
  addTranslations(originalItem?.titleTranslate);

  // 3. Add English fallback
  addTitleToGroup("英語", info?.title?.english);

  // Define preferred sort order
  const PREFERRED_ORDER = ["日本語", "ローマ字", "簡体字", "繁体字", "英語"];

  const sortedEntries = Array.from(displayGroups.entries()).sort((a, b) => {
    const idxA = PREFERRED_ORDER.indexOf(a[0]);
    const idxB = PREFERRED_ORDER.indexOf(b[0]);
    if (idxA !== -1 && idxB !== -1) return idxA - idxB;
    if (idxA !== -1) return -1;
    if (idxB !== -1) return 1;
    return a[0].localeCompare(b[0]);
  });

  if (sortedEntries.length === 0) return null;

  return (
    <div className="space-y-2 rounded-2xl border border-gray-100 bg-gray-50 p-4 text-sm dark:border-gray-700/50 dark:bg-gray-900/50">
      {sortedEntries.map(([label, titles]) => {
        if (!titles?.length) return null;

        return (
          <div key={label} className="flex gap-3">
            <span className="w-14 shrink-0 font-bold text-gray-400">
              {label}
            </span>
            <span className="text-gray-700 dark:text-gray-200">
              {titles.join(" / ")}
            </span>
          </div>
        );
      })}
    </div>
  );
}
