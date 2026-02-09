import { useEffect, useMemo, useState } from "react";
import { useLocalStorage } from "usehooks-ts";
import AttributionModal from "./components/AttributionModal";
import DetailsModal from "./components/DetailsModal";
import Footer from "./components/Footer";
import Header from "./components/Header";
import TabbedGrid from "./components/TabbedGrid";
import { STORAGE_KEY_SELECTIONS } from "./constants";
import type { AnimeItem, Config, UnifiedMetadata } from "./types";

interface Selections {
  year: string;
  season: string;
  site: string;
}

export default function App() {
  const [config, setConfig] = useState<Config | null>(null);
  const [items, setItems] = useState<AnimeItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [selections, setSelections] = useLocalStorage<Selections>(
    STORAGE_KEY_SELECTIONS,
    {
      year: "",
      season: "all",
      site: "all",
    },
  );

  const selectedYear = selections.year;
  const setSelectedYear = (year: string) =>
    setSelections((prev) => ({ ...prev, year }));

  const selectedSeason = selections.season;
  const setSelectedSeason = (season: string) =>
    setSelections((prev) => ({ ...prev, season }));

  const selectedSite = selections.site;
  const setSelectedSite = (site: string) =>
    setSelections((prev) => ({ ...prev, site }));

  const [searchQuery, setSearchQuery] = useState("");
  const [selectedAnime, setSelectedAnime] = useState<{
    title: string;
    info: UnifiedMetadata | null;
  } | null>(null);
  const [isAttributionOpen, setIsAttributionOpen] = useState(false);

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
          const { site } = prev;
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

          return { year, season, site };
        });
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      }
    }
    init();
  }, [setSelections]);

  // Fetch items
  useEffect(() => {
    if (!selectedYear) return;

    async function fetchItems() {
      setLoading(true);
      try {
        let url = `/api/items?year=${selectedYear}`;
        if (selectedSeason && selectedSeason !== "all")
          url += `&season=${selectedSeason}`;
        const response = await fetch(url);
        if (!response.ok) throw new Error("Items fetch failed");
        const data = await response.json();
        setItems(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    }
    fetchItems();
  }, [selectedYear, selectedSeason]);

  // Filter items
  const filteredItems = useMemo(() => {
    let filtered = items;

    if (selectedSite && selectedSite !== "all") {
      filtered = filtered.filter((item) =>
        item.sites?.some((s) => s.site === selectedSite),
      );
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

    return filtered;
  }, [items, selectedSite, searchQuery]);

  if (error) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gray-100 p-4 text-red-500 dark:bg-gray-900">
        <div className="text-center">
          <h1 className="mb-2 text-2xl font-bold">Error</h1>
          <p>{error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-100 text-gray-900 transition-colors dark:bg-gray-900 dark:text-gray-100">
      <Header
        config={config}
        selectedYear={selectedYear}
        setSelectedYear={setSelectedYear}
        selectedSeason={selectedSeason}
        setSelectedSeason={setSelectedSeason}
        selectedSite={selectedSite}
        setSelectedSite={setSelectedSite}
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
      />

      <main className="mx-auto max-w-7xl p-4 sm:p-6 lg:p-8">
        {loading ? (
          <div className="flex justify-center py-12">
            <div className="h-12 w-12 animate-spin rounded-full border-b-2 border-blue-500"></div>
          </div>
        ) : (
          <TabbedGrid
            items={filteredItems}
            siteMeta={config?.site_meta}
            selectedSite={selectedSite}
            onOpenModal={(title: string, info: UnifiedMetadata | null) =>
              setSelectedAnime({ title, info })
            }
          />
        )}
        <Footer onOpenAttribution={() => setIsAttributionOpen(true)} />
      </main>

      <DetailsModal
        isOpen={!!selectedAnime}
        onClose={() => setSelectedAnime(null)}
        anime={selectedAnime}
        items={items}
        siteMeta={config?.site_meta}
      />

      <AttributionModal
        isOpen={isAttributionOpen}
        onClose={() => setIsAttributionOpen(false)}
        config={config}
      />
    </div>
  );
}
