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
  const [metadata, setMetadata] = useState<UnifiedMetadata | null>(
    initialMetadata,
  );
  const [loading, setLoading] = useState(!initialMetadata && enabled);

  useEffect(() => {
    if (!enabled) {
      return;
    }

    // If we have initial metadata and it matches the current item, use it
    if (initialMetadata) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setMetadata(initialMetadata);
      setLoading(false);
      return;
    }

    let isMounted = true;
    setLoading(true);

    async function load() {
      try {
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

        const data = await fetchMetadata({
          title: item.title,
          tmdb_id: tmdbSite?.id,
          mal_id: malSite?.id,
          anilist_id: anilistSite?.id,
          year,
        });

        if (isMounted) {
          setMetadata(data || null);
        }
      } catch (err) {
        if (isDev()) {
          console.error(`Metadata error for ${item.title}:`, err);
        }
      } finally {
        if (isMounted) {
          setLoading(false);
        }
      }
    }

    load();

    return () => {
      isMounted = false;
    };
  }, [item, initialMetadata, fetchMetadata, enabled]);

  return { metadata, loading };
}
