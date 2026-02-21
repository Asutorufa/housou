import { useDeferredValue, useMemo, useState } from "react";
import useSWR from "swr";
import { useLocalStorage } from "usehooks-ts";
import { STORAGE_KEY_SELECTIONS } from "../constants";
import { useAuth } from "../contexts/AuthContext";
import type {
  AnimeItem,
  DisplayAnimeItem,
  Selections,
  Site,
  UserItemSummary,
} from "../types";
import { useConfigInitialization } from "./useConfigInitialization";

const fetcher = async (url: string) => {
  const response = await fetch(url);
  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(
      `Items fetch failed: ${response.status} ${response.statusText} - ${errorText}`,
    );
  }
  return response.json();
};

export function useAnimeData() {
  const [selections, setSelections] = useLocalStorage<Selections>(
    STORAGE_KEY_SELECTIONS,
    {
      year: "",
      season: "all",
      site: "all",
      status: "all",
    },
  );

  const { config, initError, initLoading } =
    useConfigInitialization(setSelections);

  const [searchQuery, setSearchQuery] = useState("");
  const deferredSearchQuery = useDeferredValue(searchQuery);

  const { loggedIn, apiFetch } = useAuth();

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
      ? `/api/user/status?year=${selectedYear}&season=${selectedSeason}`
      : null,
    async (url: string) => {
      const res = await apiFetch(url, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
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
      const status = parseInt(selectedStatus, 10);
      if (!isNaN(status)) {
        filtered = filtered.filter((item) => {
          const itemStatus = item.userStatus || 0;
          return itemStatus === status;
        });
      }
    }

    if (deferredSearchQuery) {
      const query = deferredSearchQuery.toLowerCase();
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

    return filtered;
  }, [items, selectedSite, selectedStatus, deferredSearchQuery]);

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
