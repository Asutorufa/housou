import { useEffect, useState } from "react";
import { useMetadata } from "../contexts/MetadataContext";
import type { DisplayAnimeItem, UnifiedMetadata } from "../types";
import { isDev } from "../utils/envUtils";

export function useSmartMetadata(
  item: DisplayAnimeItem,
  initialMetadata: UnifiedMetadata | null = null,
  enabled: boolean = true,
) {
  const { fetchMetadata } = useMetadata();
  const tmdbSite = item.sites?.find((s) => s.site === "tmdb");
  const malSite = item.sites?.find((s) => s.site === "mal");
  const anilistSite = item.sites?.find(
    (s) => s.site === "aniList" || s.site === "anilist",
  );

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
          year,
        })
      : null;

  const [fetchedResult, setFetchedResult] = useState<{
    key: string;
    metadata: UnifiedMetadata | null;
  } | null>(null);

  const metadata =
    initialMetadata ||
    (fetchedResult?.key === requestKey ? fetchedResult.metadata : null);
  const loading = requestKey !== null && fetchedResult?.key !== requestKey;

  useEffect(() => {
    if (!requestKey || fetchedResult?.key === requestKey) {
      return;
    }

    const currentRequestKey = requestKey;
    let isMounted = true;

    async function load() {
      try {
        const data = await fetchMetadata({
          title: item.title,
          tmdb_id: tmdbSite?.id,
          mal_id: malSite?.id,
          anilist_id: anilistSite?.id,
          year,
        });

        if (isMounted) {
          setFetchedResult({
            key: currentRequestKey,
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
            metadata: null,
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
    year,
  ]);

  return { metadata, loading };
}
