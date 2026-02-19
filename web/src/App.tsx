import { useState } from "react";
import { useLocation, useSearch } from "wouter";
import AttributionModal from "./components/AttributionModal";
import DetailsModal from "./components/DetailsModal";
import Footer from "./components/Footer";
import Header from "./components/Header";
import TabbedGrid from "./components/TabbedGrid";
import { useAnimeData } from "./hooks/useAnimeData";
import type { UnifiedMetadata } from "./types";

export default function App() {
  const {
    config,
    selections,
    setSelections,
    searchQuery,
    setSearchQuery,
    items,
    filteredItems,
    loading,
    error,
    mutateStatuses,
  } = useAnimeData();

  const [location, setLocation] = useLocation();
  const search = useSearch();

  const [selectedAnime, setSelectedAnime] = useState<{
    title: string;
    info: UnifiedMetadata | null;
  } | null>(null);
  const [isAttributionOpen, setIsAttributionOpen] = useState(false);

  // Sync URL "anime" param to modal state (comparison during render)
  const params = new URLSearchParams(search);
  const animeTitle = params.get("anime");

  if (animeTitle) {
    if (items.length > 0 && selectedAnime?.title !== animeTitle) {
      const item = items.find((i) => i.title === animeTitle);
      if (item) {
        // Use queueMicrotask to defer state update outside of render
        queueMicrotask(() =>
          setSelectedAnime({ title: animeTitle, info: null }),
        );
      }
    }
  } else if (selectedAnime) {
    queueMicrotask(() => setSelectedAnime(null));
  }

  const handleOpenModal = (title: string, info: UnifiedMetadata | null) => {
    // Optimistically select locally
    setSelectedAnime({ title, info });

    // Update URL
    const params = new URLSearchParams(search);
    params.set("anime", title);
    setLocation(`${location}?${params.toString()}`);
  };

  const handleCloseModal = () => {
    // Optimistically close locally
    setSelectedAnime(null);

    // Update URL
    const params = new URLSearchParams(search);
    params.delete("anime");
    setLocation(`${location}?${params.toString()}`);
  };

  const selectedYear = selections.year;
  const setSelectedYear = (year: string) =>
    setSelections((prev) => ({ ...prev, year }));

  const selectedSeason = selections.season;
  const setSelectedSeason = (season: string) =>
    setSelections((prev) => ({ ...prev, season }));

  const selectedSite = selections.site;
  const setSelectedSite = (site: string) =>
    setSelections((prev) => ({ ...prev, site }));

  const selectedStatus = selections.status || "all";
  const setSelectedStatus = (status: string) =>
    setSelections((prev) => ({ ...prev, status }));

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
        selectedStatus={selectedStatus}
        setSelectedStatus={setSelectedStatus}
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
      />

      <main className="mx-auto max-w-7xl p-4 sm:p-6 lg:p-8">
        {loading ? (
          <div className="flex justify-center py-12">
            <div className="h-12 w-12 animate-spin rounded-full border-b-2 border-blue-500"></div>
          </div>
        ) : error && items.length === 0 ? (
          <div className="flex min-h-[50vh] items-center justify-center p-4 text-red-500">
            <div className="text-center">
              <h2 className="mb-2 text-xl font-bold">
                Unable to load anime list
              </h2>
              <p className="opacity-80">{error}</p>
            </div>
          </div>
        ) : (
          <>
            {error && (
              <div className="mb-6 rounded-lg bg-red-50 p-4 text-sm text-red-800 dark:bg-red-900/20 dark:text-red-300">
                <span className="font-semibold">Note:</span> Failed to refresh
                data ({error}). Showing cached content.
              </div>
            )}
            <TabbedGrid
              items={filteredItems}
              siteMeta={config?.site_meta}
              selectedSite={selectedSite}
              onOpenModal={handleOpenModal}
            />
          </>
        )}
        <Footer onOpenAttribution={() => setIsAttributionOpen(true)} />
      </main>

      <DetailsModal
        isOpen={!!selectedAnime}
        onClose={handleCloseModal}
        anime={selectedAnime}
        items={items}
        siteMeta={config?.site_meta}
        onUpdate={() => mutateStatuses()}
      />

      <AttributionModal
        isOpen={isAttributionOpen}
        onClose={() => setIsAttributionOpen(false)}
        config={config}
      />
    </div>
  );
}
