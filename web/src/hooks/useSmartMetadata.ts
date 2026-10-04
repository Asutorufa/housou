import { useEffect, useMemo, useState } from "react";
import { useMetadata } from "../contexts/MetadataContext";
import type { DisplayAnimeItem, UnifiedMetadata } from "../types";
import { isDev } from "../utils/envUtils";

export function useSmartMetadata(
  item: Pick<DisplayAnimeItem, "title"> &
    Partial<
      Pick<DisplayAnimeItem, "sites" | "begin" | "titleTranslate" | "type">
    >,
  initialMetadata: UnifiedMetadata | null = null,
  enabled: boolean = true,
  priority: "normal" | "detail" = "normal",
) {
  const { fetchMetadata } = useMetadata();
  const [attempt, setAttempt] = useState(0);
  const tmdbSite = item.sites?.find((s) => s.site === "tmdb");
  const malSite = item.sites?.find((s) => s.site === "mal");
  const anilistSite = item.sites?.find(
    (s) => s.site === "aniList" || s.site === "anilist",
  );
  const bangumiSite = item.sites?.find(
    (s) => s.site === "bangumi" || s.site === "bgm",
  );
  const aliases = useMemo(() => {
    const translations = item.titleTranslate ?? {};
    const preferredKeys = [
      "JP",
      "ja",
      "US",
      "en",
      "CN",
      "zh-Hans",
      "TW",
      "zh-Hant",
    ];
    const preferred = new Set(preferredKeys);
    const ordered = [
      ...preferredKeys.flatMap((key) => translations[key] ?? []),
      ...Object.entries(translations)
        .filter(([key]) => !preferred.has(key))
        .flatMap(([, titles]) => titles ?? []),
    ];

    const seen = new Set([item.title.trim().toLocaleLowerCase()]);
    return ordered
      .map((title) => title.trim())
      .filter((title) => {
        const key = title.toLocaleLowerCase();
        if (!title || seen.has(key)) return false;
        seen.add(key);
        return true;
      })
      .slice(0, 6);
  }, [item.title, item.titleTranslate]);

  let year: number | undefined;
  if (item.begin) {
    const parsedYear = parseInt(item.begin.substring(0, 4));
    if (!isNaN(parsedYear)) {
      year = parsedYear;
    }
  }

  const requestKey =
    enabled && !initialMetadata
      ? JSON.stringify({
          title: item.title,
          tmdb_id: tmdbSite?.id,
          mal_id: malSite?.id,
          anilist_id: anilistSite?.id,
          bangumi_id: bangumiSite?.id,
          aliases,
          year,
          media_type: item.type,
        })
      : null;

  const [fetchedResult, setFetchedResult] = useState<{
    key: string;
    metadata: UnifiedMetadata | null;
    attempt: number;
    error?: string;
  } | null>(null);

  const metadata =
    initialMetadata ||
    (fetchedResult?.key === requestKey && fetchedResult.attempt === attempt
      ? fetchedResult.metadata
      : null);
  const loading =
    requestKey !== null &&
    (fetchedResult?.key !== requestKey || fetchedResult.attempt !== attempt);

  useEffect(() => {
    if (
      !requestKey ||
      (fetchedResult?.key === requestKey && fetchedResult.attempt === attempt)
    ) {
      return;
    }

    const currentRequestKey = requestKey;
    let isMounted = true;

    async function load() {
      try {
        const data = await fetchMetadata(
          {
            title: item.title,
            tmdb_id: tmdbSite?.id,
            mal_id: malSite?.id,
            anilist_id: anilistSite?.id,
            bangumi_id: bangumiSite?.id,
            aliases,
            year,
            media_type: item.type,
          },
          priority,
        );

        if (isMounted) {
          setFetchedResult({
            key: currentRequestKey,
            attempt,
            metadata: data || null,
          });
        }
      } catch (err) {
        if (isDev()) {
          console.error(`Metadata error for ${item.title}:`, err);
        }
        if (isMounted) {
          setFetchedResult({
            key: currentRequestKey,
            attempt,
            metadata: null,
            error:
              err instanceof Error
                ? err.message
                : "メタデータを読み込めませんでした。",
          });
        }
      }
    }

    load();

    return () => {
      isMounted = false;
    };
  }, [
    requestKey,
    fetchedResult?.key,
    fetchMetadata,
    item.title,
    tmdbSite?.id,
    malSite?.id,
    anilistSite?.id,
    bangumiSite?.id,
    aliases,
    year,
    item.type,
    attempt,
    fetchedResult?.attempt,
    priority,
  ]);

  const error =
    fetchedResult?.key === requestKey && fetchedResult.attempt === attempt
      ? fetchedResult.error
      : undefined;
  return {
    metadata,
    loading,
    error,
    retry: () => setAttempt((value) => value + 1),
  };
}
