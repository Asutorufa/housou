import { useEffect, useMemo, useState } from "react";
import useSWR from "swr";
import { useLocalStorage } from "usehooks-ts";
import { STORAGE_KEY_SELECTIONS } from "../constants";
import { useAuth } from "../contexts/AuthContext";
import type {
  AnimeItem,
  Config,
  DisplayAnimeItem,
  Selections,
  Site,
  UserItemSummary,
} from "../types";

const fetcher = async (url: string) => {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(
      `Items fetch failed: ${response.status} ${response.statusText}`,
    );
  }
  return response.json();
};

export function useAnimeData() {
  const [config, setConfig] = useState<Config | null>(null);
  const [initError, setInitError] = useState<string | null>(null);
  const [initLoading, setInitLoading] = useState(true);

  const [selections, setSelections] = useLocalStorage<Selections>(
    STORAGE_KEY_SELECTIONS,
    {
      year: "",
      season: "all",
      site: "all",
      status: "all",
    },
  );

  const [searchQuery, setSearchQuery] = useState("");

  const { loggedIn, apiFetch } = useAuth();

  // Initialization
  useEffect(() => {
    async function init() {
      try {
        const response = await fetch(`/api/config?v=${new Date().getTime()}`);
        if (!response.ok) throw new Error("Config fetch failed");
        const data: Config = await response.json();
        setConfig(data);

        // Validate or set defaults
        setSelections((prev) => {
          let { year, season } = prev;
          const { site, status } = prev;
          const isYearValid = year && data.years.includes(parseInt(year));

          if (!isYearValid) {
            // Apply defaults
            const currentYear = new Date().getFullYear().toString();
            if (data.years.includes(parseInt(currentYear))) {
              year = currentYear;
            } else if (data.years.length > 0) {
              year = data.years[data.years.length - 1].toString();
            } else {
              year = "";
            }

            // Get current season
            const seasons = ["Winter", "Spring", "Summer", "Autumn"];
            season = seasons[Math.floor(new Date().getMonth() / 3)];
          }

          return { year, season, site, status: status || "all" };
        });
      } catch (err) {
        setInitError(err instanceof Error ? err.message : String(err));
      } finally {
        setInitLoading(false);
      }
    }
    init();
  }, [setSelections]);

  const selectedYear = selections.year;
  const selectedSeason = selections.season;
  const selectedSite = selections.site;
  const selectedStatus = selections.status || "all";

  // Use SWR directly for fetching items
  const itemsUrl = useMemo(() => {
    if (!selectedYear) return null;

    const params = new URLSearchParams({ year: selectedYear });
    if (selectedSeason && selectedSeason !== "all") {
      params.append("season", selectedSeason);
    }

    return `/api/items?${params.toString()}`;
  }, [selectedYear, selectedSeason]);

  const {
    data: fetchedItems,
    error: itemsError,
    isLoading: itemsLoading,
  } = useSWR<AnimeItem[]>(itemsUrl, fetcher);

  // Fetch user statuses separately
  const { data: userStatuses, mutate: mutateStatuses } = useSWR<
    Record<string, UserItemSummary>
  >(
    loggedIn && config?.auth_enabled && itemsUrl && fetchedItems?.length
      ? ["/api/user/status", itemsUrl]
      : null,
    async ([url]) => {
      const res = await apiFetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify(fetchedItems!.map((i) => i.title)),
      });
      if (!res.ok) {
        throw new Error("Failed to fetch statuses");
      }
      return res.json();
    },
  );

  const items = useMemo((): DisplayAnimeItem[] => {
    if (!fetchedItems) return [];
    if (!userStatuses) return fetchedItems;

    return fetchedItems.map((item) => {
      const summary = userStatuses[item.title];
      if (summary) {
        return {
          ...item,
          userStatus: summary.status,
          userScore: summary.score ?? undefined,
        };
      }
      return item;
    });
  }, [fetchedItems, userStatuses]);

  // Filter items
  const filteredItems = useMemo(() => {
    let filtered = items;

    if (selectedSite && selectedSite !== "all") {
      filtered = filtered.filter((item) =>
        item.sites?.some((s: Site) => s.site === selectedSite),
      );
    }

    if (selectedStatus && selectedStatus !== "all") {
      const status = parseInt(selectedStatus);
      filtered = filtered.filter((item) => {
        const itemStatus = item.userStatus || 0;
        return itemStatus === status;
      });
    }

    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter((item) => {
        if (item.title.toLowerCase().includes(query)) return true;
        if (item.titleTranslate) {
          return Object.values(item.titleTranslate).some((ts) =>
            ts?.some((t) => t.toLowerCase().includes(query)),
          );
        }
        return false;
      });
    }

    console.log(
      `Filtered items: ${filtered.length}/${items.length} (Status: ${selectedStatus})`,
    );
    return filtered;
  }, [items, selectedSite, selectedStatus, searchQuery]);

  const loading = initLoading || itemsLoading;

  const error =
    initError ||
    (itemsError
      ? itemsError instanceof Error
        ? itemsError.message
        : String(itemsError)
      : null);

  return {
    config,
    selections,
    setSelections,
    searchQuery,
    setSearchQuery,
    filteredItems,
    items,
    loading,
    error,
    mutateStatuses,
  };
}
